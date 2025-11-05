use std::net::IpAddr;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::time::Instant;
use lru::LruCache;

#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    pub enabled: bool,
    pub qps: f64,
    pub burst: f64,
    pub max_ips: usize,
}

#[derive(Clone, Debug)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

/// Per-key token-bucket rate limiter with LRU-bounded state
pub struct RateLimiter {
    config: RateLimiterConfig,
    buckets: Mutex<LruCache<String, Bucket>>, // keyed by IP string
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        let cap = NonZeroUsize::new(config.max_ips.max(1)).unwrap();
        Self {
            config,
            buckets: Mutex::new(LruCache::new(cap)),
        }
    }

    fn key_for(ip: IpAddr) -> String {
        ip.to_string()
    }

    /// Compatibility API used by middleware: override QPS per call; burst from config
    pub fn check_rate_limit(&self, ip: IpAddr, qps_override: f64) -> bool {
        if !self.config.enabled {
            return true;
        }
        let qps = if qps_override > 0.0 { qps_override } else { self.config.qps };
        let burst = self.config.burst.max(0.0);
        let key = Self::key_for(ip);
        let (allow, _retry) = self.check_key(&key, qps, burst);
        allow
    }

    /// General check for an arbitrary key; returns (allow, retry_after_secs)
    pub fn check_key(&self, key: &str, qps: f64, burst: f64) -> (bool, u64) {
        if !self.config.enabled {
            return (true, 0);
        }
        if qps <= 0.0 || burst <= 0.0 {
            return (false, 1);
        }

        let now = Instant::now();
        let mut map = self.buckets.lock().expect("rate limiter mutex poisoned");

        if let Some(bucket) = map.get_mut(key) {
            // Refill tokens by elapsed time
            let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
            bucket.tokens = (bucket.tokens + elapsed * qps).min(burst);
            bucket.last_refill = now;

            if bucket.tokens >= 1.0 {
                bucket.tokens -= 1.0;
                (true, 0)
            } else {
                let needed = 1.0 - bucket.tokens;
                let secs = (needed / qps).ceil() as u64;
                (false, secs.max(1))
            }
        } else {
            // New bucket: allow immediately if burst allows at least one token
            let allow = burst >= 1.0;
            let tokens = if allow { (burst - 1.0).max(0.0) } else { 0.0 };
            map.put(key.to_string(), Bucket { tokens, last_refill: now });
            if allow { (true, 0) } else { (false, 1) }
        }
    }

    /// Compute Retry-After for a key at current state and qps
    pub fn compute_retry_after(&self, key: &str, qps: f64) -> u64 {
        if !self.config.enabled {
            return 0;
        }
        if qps <= 0.0 {
            return 1;
        }

        let now = Instant::now();
        let mut map = self.buckets.lock().expect("rate limiter mutex poisoned");

        if let Some(bucket) = map.get_mut(key) {
            let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
            bucket.tokens = (bucket.tokens + elapsed * qps).min(self.config.burst.max(0.0));
            bucket.last_refill = now;

            if bucket.tokens >= 1.0 {
                0
            } else {
                let needed = 1.0 - bucket.tokens;
                (needed / qps).ceil() as u64
            }
        } else {
            1
        }
    }
}
