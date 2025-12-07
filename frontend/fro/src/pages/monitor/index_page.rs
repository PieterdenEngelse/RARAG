use crate::{api, app::Route, components::monitor::*};
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;

#[derive(Clone)]
struct IndexState {
    loading: bool,
    error: Option<String>,
    index_in_ram: bool,
    mode: String,
    warning: Option<String>,
    total_documents: usize,
    total_vectors: usize,
    last_action: Option<String>,
}

impl Default for IndexState {
    fn default() -> Self {
        Self {
            loading: true,
            error: None,
            index_in_ram: false,
            mode: "Unknown".into(),
            warning: None,
            total_documents: 0,
            total_vectors: 0,
            last_action: None,
        }
    }
}

#[component]
pub fn MonitorIndex() -> Element {
    let state = use_signal(IndexState::default);

    {
        let mut state = state.clone();
        use_future(move || async move {
            loop {
                match api::fetch_index_info().await {
                    Ok(info) => {
                        let last_action = state.read().last_action.clone();
                        state.set(IndexState {
                            loading: false,
                            error: None,
                            index_in_ram: info.index_in_ram,
                            mode: info.mode,
                            warning: info.warning,
                            total_documents: info.total_documents,
                            total_vectors: info.total_vectors,
                            last_action,
                        });
                    }
                    Err(err) => {
                        let previous = state.read().clone();
                        state.set(IndexState {
                            loading: false,
                            error: Some(err),
                            ..previous
                        });
                    }
                }

                TimeoutFuture::new(10_000).await;
            }
        });
    }

    let trigger_reindex = {
        let state = state.clone();
        move |_| {
            let mut state = state.clone();
            spawn(async move {
                state.write().last_action = Some("Reindex running…".into());
                match api::reindex().await {
                    Ok(_) => state.write().last_action = Some("Reindex started".into()),
                    Err(err) => state.write().last_action = Some(format!("Failed: {}", err)),
                }
            });
        }
    };

    let snapshot = state.read();

    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Monitor", Some(Route::MonitorOverview {})),
                    BreadcrumbItem::new("Index", None),
                ],
            }

            NavTabs { active: Route::MonitorIndex {} }

            RowHeader {
                title: "Index Statistics".into(),
                description: Some("Live snapshot from /index/info".into()),
            }
            if snapshot.loading {
                div { class: "text-sm text-gray-400", "Loading index info..." }
            } else if let Some(err) = &snapshot.error {
                div { class: "text-sm text-red-400", "Failed to load index info: {err}" }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                    StatCard {
                        title: "Documents".into(),
                        value: format!("{}", snapshot.total_documents).into(),
                        unit: None,
                    }
                    StatCard {
                        title: "Vectors".into(),
                        value: format!("{}", snapshot.total_vectors).into(),
                        unit: None,
                    }
                    StatCard {
                        title: "Mode".into(),
                        value: snapshot.mode.clone().into(),
                        unit: None,
                        trend: snapshot.warning.clone().map(|w| w.into()),
                    }
                    StatCard {
                        title: "Storage".into(),
                        value: (if snapshot.index_in_ram { "RAM" } else { "Disk" }).to_string().into(),
                        unit: Some(if snapshot.index_in_ram { "fast" } else { "standard" }.into()),
                    }
                }

                if let Some(warning) = &snapshot.warning {
                    div { class: "text-xs text-yellow-400", "{warning}" }
                }
            }

            Panel { title: Some("Reindex".into()), refresh: Some("manual".into()),
                div { class: "text-sm text-gray-300 space-y-2",
                    p { "Trigger a manual reindex to refresh the Tantivy/embedding data." }
                    if let Some(action) = &snapshot.last_action {
                        div { class: "text-xs text-gray-400", "{action}" }
                    }
                    button {
                        class: "px-4 py-2 rounded bg-indigo-600 text-white hover:bg-indigo-500",
                        onclick: trigger_reindex,
                        "Trigger Reindex"
                    }
                }
            }
        }
    }
}
