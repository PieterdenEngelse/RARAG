use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::Mutex;

/// Single HTTP request sample captured by the trace middleware.
#[derive(Clone, Debug)]
struct RequestSample {
    ts: DateTime<Utc>,
    latency_ms: f64,
    is_error: bool,
    status_class: String,
}

/// Chart point exposed to the frontend.
#[derive(Clone, Debug, Serialize)]
pub struct RequestChartPoint {
    /// UNIX timestamp (seconds since epoch)
    pub ts: i64,
    pub latency_ms: f64,
}

/// Snapshot returned to the frontend for summary + chart.
#[derive(Clone, Debug, Serialize, Default)]
pub struct LatencyBreakdown {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct StatusBreakdown {
    pub success_rate: f64,
    pub client_error_rate: f64,
    pub server_error_rate: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestsSnapshot {
    pub request_rate_rps: f64,
    pub latency_p95_ms: f64,
    pub error_rate_percent: f64,
    pub latency_breakdown: LatencyBreakdown,
    pub status_breakdown: StatusBreakdown,
    pub points: Vec<RequestChartPoint>,
}

static REQUEST_SAMPLES: Lazy<Mutex<VecDeque<RequestSample>>> = Lazy::new(|| {
    // Reserve some space but let it grow as needed up to a reasonable cap.
    Mutex::new(VecDeque::with_capacity(1024))
});

// Keep at most this many seconds of samples for UI purposes.
const MAX_WINDOW_SECS: i64 = 5 * 60; // 5 minutes

/// Record a single HTTP request sample for UI metrics.
///
/// Called from the trace middleware after each completed request.
pub fn record_http_request(latency_ms: f64, is_error: bool, status_class: &str) {
    let mut buf = REQUEST_SAMPLES.lock().unwrap();
    let now = Utc::now();

    buf.push_back(RequestSample {
        ts: now,
        latency_ms,
        is_error,
        status_class: status_class.to_string(),
    });

    // Drop samples older than MAX_WINDOW_SECS to keep memory bounded.
    let cutoff = now - chrono::Duration::seconds(MAX_WINDOW_SECS);
    while let Some(front) = buf.front() {
        if front.ts < cutoff {
            buf.pop_front();
        } else {
            break;
        }
    }

    // Optional hard cap as a safety net.
    const HARD_CAP: usize = 5000;
    if buf.len() > HARD_CAP {
        let excess = buf.len() - HARD_CAP;
        for _ in 0..excess {
            buf.pop_front();
        }
    }
}

/// Compute a snapshot for the Requests dashboard from recent samples.
///
/// - Summary (rate, p95 latency, error%) is computed over roughly the last 60 seconds.
/// - Chart points cover the last MAX_WINDOW_SECS seconds.
pub fn get_requests_snapshot() -> RequestsSnapshot {
    let buf = REQUEST_SAMPLES.lock().unwrap();

    if buf.is_empty() {
        return RequestsSnapshot {
            request_rate_rps: 0.0,
            latency_p95_ms: 0.0,
            error_rate_percent: 0.0,
            latency_breakdown: LatencyBreakdown::default(),
            status_breakdown: StatusBreakdown::default(),
            points: Vec::new(),
        };
    }

    let now = Utc::now();
    let summary_window = chrono::Duration::seconds(60);
    let summary_cutoff = now - summary_window;

    let mut summary_latencies: Vec<f64> = Vec::new();
    let mut summary_total = 0usize;
    let mut summary_errors = 0usize;
    let mut client_errors = 0usize;
    let mut server_errors = 0usize;

    // Collect summary window stats and chart points.
    let mut points: Vec<RequestChartPoint> = Vec::new();
    let chart_cutoff = now - chrono::Duration::seconds(MAX_WINDOW_SECS);

    for s in buf.iter() {
        // Chart: keep everything within MAX_WINDOW_SECS
        if s.ts >= chart_cutoff {
            points.push(RequestChartPoint {
                ts: s.ts.timestamp(),
                latency_ms: s.latency_ms,
            });
        }

        // Summary: only last 60 seconds (or all if window is smaller)
        if s.ts >= summary_cutoff {
            summary_total += 1;
            if s.is_error {
                summary_errors += 1;
            }
            match s.status_class.as_str() {
                "4xx" => client_errors += 1,
                "5xx" => server_errors += 1,
                _ => {}
            }
            summary_latencies.push(s.latency_ms);
        }
    }

    if summary_total == 0 {
        // Fallback: use entire buffer as summary window.
        summary_total = buf.len();
        summary_latencies = buf.iter().map(|s| s.latency_ms).collect::<Vec<f64>>();
        summary_errors = buf.iter().filter(|s| s.is_error).count();
        client_errors = buf.iter().filter(|s| s.status_class == "4xx").count();
        server_errors = buf.iter().filter(|s| s.status_class == "5xx").count();
    }

    let latency_snapshot = summary_latencies.clone();

    let request_rate_rps = if summary_total == 0 {
        0.0
    } else {
        // Approximate window length in seconds based on timestamps.
        let first_ts = buf.front().unwrap().ts;
        let elapsed = (now - first_ts).num_seconds().max(1) as f64;
        (summary_total as f64) / elapsed
    };

    let error_rate_percent = if summary_total == 0 {
        0.0
    } else {
        (summary_errors as f64) * 100.0 / (summary_total as f64)
    };

    let latency_p95_ms = if summary_latencies.is_empty() {
        0.0
    } else {
        let mut v = summary_latencies;
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let idx = ((v.len() as f64) * 0.95).ceil() as usize - 1;
        let idx = idx.clamp(0, v.len() - 1);
        v[idx]
    };

    let latency_breakdown = compute_latency_breakdown(&latency_snapshot);
    let status_breakdown = compute_status_breakdown(summary_total, client_errors, server_errors);

    RequestsSnapshot {
        request_rate_rps,
        latency_p95_ms,
        error_rate_percent,
        latency_breakdown,
        status_breakdown,
        points,
    }
}

fn compute_latency_breakdown(latencies: &[f64]) -> LatencyBreakdown {
    if latencies.is_empty() {
        return LatencyBreakdown::default();
    }

    let mut values = latencies.to_vec();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let percentile = |p: f64| -> f64 {
        let idx = ((values.len() as f64) * p).ceil() as usize - 1;
        values[idx.clamp(0, values.len() - 1)]
    };

    LatencyBreakdown {
        p50_ms: percentile(0.50),
        p95_ms: percentile(0.95),
        p99_ms: percentile(0.99),
    }
}

fn compute_status_breakdown(total: usize, client: usize, server: usize) -> StatusBreakdown {
    if total == 0 {
        return StatusBreakdown::default();
    }

    let total_f = total as f64;
    let client_f = client as f64;
    let server_f = server as f64;

    StatusBreakdown {
        success_rate: ((total as f64 - client_f - server_f) / total_f) * 100.0,
        client_error_rate: (client_f / total_f) * 100.0,
        server_error_rate: (server_f / total_f) * 100.0,
    }
}
