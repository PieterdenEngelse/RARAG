use crate::{api, app::Route, components::monitor::*};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

#[derive(Clone, Default)]
struct RateLimitState {
    loading: bool,
    error: Option<String>,
    data: Option<api::RateLimitInfoResponse>,
}

#[component]
pub fn MonitorRateLimits() -> Element {
    let state = use_signal(|| RateLimitState {
        loading: true,
        ..Default::default()
    });

    {
        let mut state = state.clone();
        use_future(move || async move {
            loop {
                match api::fetch_rate_limit_info().await {
                    Ok(resp) => state.set(RateLimitState {
                        loading: false,
                        error: None,
                        data: Some(resp),
                    }),
                    Err(err) => {
                        let previous = state.read().data.clone();
                        state.set(RateLimitState {
                            loading: false,
                            error: Some(err),
                            data: previous,
                        });
                    }
                }
                TimeoutFuture::new(5_000).await;
            }
        });
    }

    let snapshot = state.read().clone();
    let drop_rows = snapshot.data.as_ref().map(|d| build_drop_rows(d));
    let drop_counts = snapshot.data.as_ref().map(|d| build_drop_counts(d));

    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Monitor", Some(Route::MonitorOverview {})),
                    BreadcrumbItem::new("Rate Limits", None),
                ],
            }

            NavTabs { active: Route::MonitorRateLimits {} }

            Panel { title: Some("Summary".into()), refresh: Some("5s".into()),
                if snapshot.loading {
                    div { class: "text-gray-400 text-sm", "Loading rate-limit stats…" }
                } else if let Some(err) = snapshot.error {
                    div { class: "text-red-400 text-sm", "Failed to load stats: {err}" }
                } else if let Some(data) = snapshot.data.clone() {
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        StatCard {
                            title: "Total Drops".into(),
                            value: data.total_drops.to_string().into(),
                            unit: None,
                        }
                        StatCard {
                            title: "Active Keys".into(),
                            value: data.limiter_state.active_keys.to_string().into(),
                            unit: Some(format!("/{}", data.limiter_state.capacity).into()),
                        }
                        StatCard {
                            title: "Limiter".into(),
                            value: (if data.config.enabled { "On" } else { "Off" }).into(),
                            unit: None,
                        }
                    }

                    if let Some(values) = drop_counts.clone() {
                        if !values.is_empty() {
                            Panel { title: Some("Drop Trend".into()), refresh: Some("5s".into()),
                                ChartPlaceholder {
                                    values,
                                    label: "Drops per route (top 5)".to_string(),
                                    unit: " drops".to_string(),
                                }
                            }
                        }
                    }

                    Panel { title: Some("Drops by Route".into()), refresh: Some("5s".into()),
                        if drop_rows
                            .as_ref()
                            .map(|rows| rows.is_empty())
                            .unwrap_or(true)
                        {
                            div { class: "text-gray-500 text-sm", "No drops recorded yet." }
                        } else {
                            DataTable {
                                headers: vec!["Route".into(), "Drops".into()],
                                rows: drop_rows.clone().unwrap_or_default(),
                            }
                        }
                    }

                    Panel { title: Some("Configuration".into()), refresh: None,
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                            DataTable {
                                headers: vec!["Parameter".into(), "Value".into()],
                                rows: vec![
                                    vec!["Trust Proxy".into(), yes_no(data.config.trust_proxy)],
                                    vec!["Search QPS".into(), format_float(data.config.search_qps).into()],
                                    vec!["Search Burst".into(), format_float(data.config.search_burst).into()],
                                    vec!["Upload QPS".into(), format_float(data.config.upload_qps).into()],
                                    vec!["Upload Burst".into(), format_float(data.config.upload_burst).into()],
                                ],
                            }
                            DataTable {
                                headers: vec!["Exempt Prefixes".into()],
                                rows: data.config.exempt_prefixes.iter().map(|p| vec![p.clone().into()]).collect(),
                            }
                        }

                        div { class: "mt-4",
                            RowHeader {
                                title: "Custom Rules".into(),
                                description: Some("As loaded from RATE_LIMIT_ROUTES".into()),
                            }
                            if data.config.rules.is_empty() {
                                div { class: "text-gray-500 text-sm", "No custom rules configured." }
                            } else {
                                DataTable {
                                    headers: vec!["Pattern".into(), "Match".into(), "QPS".into(), "Burst".into(), "Label".into()],
                                    rows: data.config.rules.iter().map(|rule| {
                                        let pattern = rule.get("pattern").and_then(|v| v.as_str()).unwrap_or("-");
                                        let match_kind = rule.get("match_kind").and_then(|v| v.as_str()).unwrap_or("-");
                                        let qps = rule.get("qps").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let burst = rule.get("burst").and_then(|v| v.as_f64()).unwrap_or(0.0);
                                        let label = rule.get("label").and_then(|v| v.as_str()).unwrap_or("-");
                                        vec![
                                            pattern.into(),
                                            match_kind.into(),
                                            format_float(qps).into(),
                                            format_float(burst).into(),
                                            label.into(),
                                        ]
                                    }).collect(),
                                }
                            }
                        }
                    }
                } else {
                    div { class: "text-gray-400 text-sm", "No data yet." }
                }
            }
        }
    }
}

fn yes_no(flag: bool) -> String {
    if flag { "Yes".into() } else { "No".into() }
}

fn format_float(value: f64) -> String {
    format!("{:.2}", value)
}

fn build_drop_rows(data: &api::RateLimitInfoResponse) -> Vec<Vec<String>> {
    let mut entries = data.drops_by_route.clone();
    entries.sort_by(|a, b| b.drops.cmp(&a.drops));
    entries
        .into_iter()
        .map(|entry| vec![entry.route, entry.drops.to_string()])
        .collect()
}

fn build_drop_counts(data: &api::RateLimitInfoResponse) -> Vec<f64> {
    let mut entries = data.drops_by_route.clone();
    entries.sort_by(|a, b| b.drops.cmp(&a.drops));
    entries
        .into_iter()
        .take(5)
        .map(|entry| entry.drops as f64)
        .collect()
}
