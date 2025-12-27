use crate::{api, app::Route, components::monitor::*};
use dioxus::prelude::*;

#[derive(Clone, Debug)]
struct ChunkPreset {
    key: String,
    target_size: usize,
    min_size: usize,
    max_size: usize,
    overlap: usize,
    semantic_threshold: f32,
}

#[derive(Clone, Debug)]
struct ChunkCategory {
    description: &'static str,
    presets: Vec<ChunkPreset>,
}

fn chunk_categories() -> Vec<ChunkCategory> {
    vec![
        ChunkCategory {
            description: "Short exchanges:",
            presets: vec![
                ChunkPreset {
                    key: "256".into(),
                    target_size: 256,
                    min_size: 128,
                    max_size: 384,
                    overlap: 32,
                    semantic_threshold: 0.78,
                },
                ChunkPreset {
                    key: "384".into(),
                    target_size: 384,
                    min_size: 192,
                    max_size: 512,
                    overlap: 48,
                    semantic_threshold: 0.78,
                },
            ],
        },
        ChunkCategory {
            description: "Functions are self-contained:",
            presets: vec![
                ChunkPreset {
                    key: "512".into(),
                    target_size: 512,
                    min_size: 256,
                    max_size: 768,
                    overlap: 64,
                    semantic_threshold: 0.78,
                },
            ],
        },
        ChunkCategory {
            description: "Concepts need context:",
            presets: vec![
                ChunkPreset {
                    key: "768".into(),
                    target_size: 768,
                    min_size: 384,
                    max_size: 1024,
                    overlap: 96,
                    semantic_threshold: 0.78,
                },
            ],
        },
        ChunkCategory {
            description: "Clauses need full context:",
            presets: vec![
                ChunkPreset {
                    key: "1024".into(),
                    target_size: 1024,
                    min_size: 512,
                    max_size: 1536,
                    overlap: 128,
                    semantic_threshold: 0.82,
                },
            ],
        },
    ]
}

fn all_presets() -> Vec<ChunkPreset> {
    chunk_categories()
        .into_iter()
        .flat_map(|c| c.presets)
        .collect()
}

#[component]
pub fn Config() -> Element {
    let categories = chunk_categories();
    let presets = all_presets();

    let mut selected_key = use_signal(|| "256".to_string());
    let mut commit_status = use_signal(|| Option::<String>::None);
    let mut committing = use_signal(|| false);
    let mut last_job_id = use_signal(|| Option::<String>::None);

    let on_commit = {
        let presets = presets.clone();
        move |_| {
            let preset_key = selected_key();
            let Some(preset) = presets.iter().find(|p| p.key == preset_key) else {
                commit_status.set(Some("Unknown chunk preset".into()));
                return;
            };
            let preset = preset.clone();
            committing.set(true);
            commit_status.set(Some("Applying settings…".into()));
            spawn(async move {
                let payload = api::ChunkCommitRequest {
                    target_size: preset.target_size,
                    min_size: preset.min_size,
                    max_size: preset.max_size,
                    overlap: preset.overlap,
                    semantic_similarity_threshold: Some(preset.semantic_threshold),
                };
                match api::commit_chunk_config(&payload).await {
                    Ok(resp) => {
                        if resp.reindex_job_id.is_some() {
                            last_job_id.set(resp.reindex_job_id.clone());
                        }
                        commit_status.set(Some(resp.message));
                    }
                    Err(err) => {
                        commit_status.set(Some(format!("Commit failed: {}", err)));
                    }
                }
                committing.set(false);
            });
        }
    };

    rsx! {
        div { class: "space-y-6",
            Breadcrumb {
                items: vec![
                    BreadcrumbItem::new("Home", Some(Route::Home {})),
                    BreadcrumbItem::new("Config", Some(Route::Config {})),
                ],
            }

            RowHeader {
                title: "RAG".into(),
                description: Some("RAG subsystem status.".into()),
            }
            Panel { title: Some("RAG".into()), refresh: None,
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "rounded p-4 bg-gray-800 border border-gray-700 flex flex-col gap-3",
                            div {
                                span { class: "text-base text-gray-200 font-semibold", "Chunk" }
                                span { class: "text-base text-gray-200 font-semibold", "Size - Target" }
                            }
                            div { class: "flex flex-wrap gap-x-4 gap-y-2 text-xs text-gray-200",
                                for category in categories.iter() {
                                    {
                                        let desc = category.description;
                                        let cat_presets = category.presets.clone();
                                        rsx! {
                                            div { class: "flex items-center gap-1",
                                                span { class: "text-gray-300", "{desc}" }
                                                for preset in cat_presets.iter() {
                                                    {
                                                        let key = preset.key.clone();
                                                        let key_for_check = preset.key.clone();
                                                        let key_for_set = preset.key.clone();
                                                        rsx! {
                                                            label { class: "flex items-center gap-1 ml-1",
                                                                input {
                                                                    r#type: "radio",
                                                                    name: "chunk-size",
                                                                    value: "{key}",
                                                                    checked: selected_key() == key_for_check,
                                                                    class: "radio radio-xs",
                                                                    style: "border-color:#fff;",
                                                                    onchange: move |_| {
                                                                        selected_key.set(key_for_set.clone());
                                                                    }
                                                                }
                                                                span { "{key}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            div { class: "flex items-center justify-between pt-2",
                                div { class: "flex items-center gap-3",
                                    button {
                                        class: "btn btn-primary btn-sm",
                                        onclick: on_commit.clone(),
                                        disabled: committing(),
                                        if committing() { "Applying…" } else { "Commit" }
                                    }
                                    if let Some(status) = commit_status() {
                                        span { class: "text-xs text-gray-200", "{status}" }
                                    }
                                }
                                button {
                                    class: "btn btn-outline btn-sm",
                                    "Info"
                                }
                            }
                            if let Some(job_id) = last_job_id() {
                                div { class: "text-[0.7rem] text-gray-400",
                                    "Reindex job ID: {job_id} (monitor via /reindex/status/{job_id})"
                                }
                            }
                        }
                        HealthCard { name: "Chunk-Size Overlapping".into(), status: "Healthy".into(), detail: Some("Ready".into()) }
                        HealthCard { name: "Chunker".into(), status: "Ready".into(), detail: Some("384 tokens".into()) }
                        HealthCard { name: "Documents".into(), status: "--".into(), detail: Some("Uploaded".into()) }
                    }
                }

            RowHeader {
                title: "Agent".into(),
                description: Some("Agent runtime status.".into()),
            }
            Panel { title: Some("Agent".into()), refresh: None,
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        HealthCard { name: "Memory".into(), status: "Active".into(), detail: Some("SQLite".into()) }
                        HealthCard { name: "Tools".into(), status: "3".into(), detail: Some("Enabled".into()) }
                        HealthCard { name: "LLM".into(), status: "phi".into(), detail: Some("Local".into()) }
                        HealthCard { name: "Usage".into(), status: "--".into(), detail: Some("Recent".into()) }
                    }
                }
        }
    }
}
