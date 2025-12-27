use crate::{api, app::Route, components::monitor::*};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

#[derive(Clone, Default)]
struct OverviewState {
    loading: bool,
    error: Option<String>,
    health_status: Option<String>,
    documents: Option<usize>,
    vectors: Option<usize>,
    request_rate_rps: Option<f64>,
    latency_p95_ms: Option<f64>,
    error_rate_percent: Option<f64>,
}

#[component]
pub fn MonitorOverview() -> Element {
    let state = use_signal(OverviewState::default);

    {
        let mut state = state.clone();
        use_future(move || async move {
            loop {
                let health = api::health_check().await;
                let requests = api::fetch_requests_snapshot().await;

                match (health, requests) {
                    (Ok(h), Ok(r)) => {
                        state.set(OverviewState {
                            loading: false,
                            error: None,
                            health_status: Some(h.status),
                            documents: h.documents,
                            vectors: h.vectors,
                            request_rate_rps: Some(r.request_rate_rps),
                            latency_p95_ms: Some(r.latency_p95_ms),
                            error_rate_percent: Some(r.error_rate_percent),
                        });
                    }
                    (Ok(h), Err(req_err)) => {
                        let previous = state.read().clone();
                        state.set(OverviewState {
                            loading: false,
                            error: Some(format!("Failed to load request stats: {}", req_err)),
                            health_status: Some(h.status),
                            documents: h.documents,
                            vectors: h.vectors,
                            ..previous
                        });
                    }
                    (Err(err), _) => {
                        let previous = state.read().clone();
                        state.set(OverviewState {
                            loading: false,
                            error: Some(err),
                            ..previous
                        });
                    }
                }

                TimeoutFuture::new(5_000).await;
            }
        });
    }

    let snapshot = state.read().clone();

    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Monitor", Some(Route::MonitorOverview {})),
                ],
            }

            NavTabs { active: Route::MonitorOverview {} }

            Panel { title: Some("System Health".into()), refresh: Some("5s".into()),
                if snapshot.loading {
                    div { class: "text-gray-400 text-sm", "Loading health status…" }
                } else if let Some(err) = snapshot.error.clone() {
                    div { class: "text-red-400 text-sm", "Failed to load health: {err}" }
                } else {
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        HealthCard {
                            name: "API".into(),
                            status: snapshot.health_status.clone().unwrap_or_else(|| "unknown".into()).into(),
                            detail: Some("Actix".into()),
                        }
                        HealthCard {
                            name: "Documents".into(),
                            status: snapshot.documents.map(|d| d.to_string()).unwrap_or_else(|| "--".into()).into(),
                            detail: Some("Indexed".into()),
                        }
                        HealthCard {
                            name: "Vectors".into(),
                            status: snapshot.vectors.map(|v| v.to_string()).unwrap_or_else(|| "--".into()).into(),
                            detail: Some("Embedding store".into()),
                        }
                    }
                }
            }

            RowHeader {
                title: "Key Metrics".into(),
                description: Some("Live request stats refreshed every 5s.".into()),
            }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard {
                    title: "Requests/sec".into(),
                    value: snapshot
                        .request_rate_rps
                        .map(|v| format!("{:.2}", v))
                        .unwrap_or_else(|| "--".into())
                        .into(),
                    unit: Some("req/s".into()),
                }
                StatCard {
                    title: "p95 Latency".into(),
                    value: snapshot
                        .latency_p95_ms
                        .map(|v| format!("{:.1}", v))
                        .unwrap_or_else(|| "--".into())
                        .into(),
                    unit: Some("ms".into()),
                }
                StatCard {
                    title: "Error Rate".into(),
                    value: snapshot
                        .error_rate_percent
                        .map(|v| format!("{:.2}", v))
                        .unwrap_or_else(|| "--".into())
                        .into(),
                    unit: Some("%".into()),
                }
            }

            RowHeader {
                title: "Quick Actions".into(),
                description: None,
            }
            div { class: "flex flex-wrap gap-3",
                button { class: "px-4 py-2 rounded bg-indigo-600 text-white", "Trigger Reindex" }
                button { class: "px-4 py-2 rounded bg-gray-700 text-gray-200", "Clear Cache" }
                button {
                    class: "px-4 py-2 rounded border border-teal-400 text-teal-400 hover:bg-teal-500/10",
                    "View Grafana"
                }
            }
        }
    }
}
