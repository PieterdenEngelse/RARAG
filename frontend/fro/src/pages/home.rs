use crate::api;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub context: Option<String>, // RAG context used (if any)
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    #[serde(default)]
    #[allow(dead_code)]
    done: bool,
}

const OLLAMA_URL: &str = "http://127.0.0.1:11434/api/generate";

#[component]
pub fn Home() -> Element {
    let mut messages = use_signal(|| Vec::<ChatMessage>::new());
    let mut input_text = use_signal(|| String::new());
    let mut is_loading = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);
    let mut selected_model = use_signal(|| "phi:latest".to_string());

    // File upload state
    let mut show_upload_panel = use_signal(|| false);
    let mut documents = use_signal(|| Vec::<String>::new());
    let mut upload_status = use_signal(|| Option::<String>::None);
    let mut is_uploading = use_signal(|| false);

    // RAG toggle
    let mut rag_enabled = use_signal(|| true);

    // Info panel state
    let mut show_info = use_signal(|| false);

    // Models actually installed in Ollama
    let available_models = vec!["phi:latest"];

    // Load documents on mount
    use_effect(move || {
        spawn(async move {
            match api::list_documents().await {
                Ok(resp) => documents.set(resp.documents),
                Err(_) => {} // Silently fail
            }
        });
    });

    // Build prompt with RAG context - returns (prompt, context_summary)
    async fn build_rag_prompt(query: &str, rag_enabled: bool) -> (String, Option<String>) {
        if !rag_enabled {
            return (query.to_string(), None);
        }

        // Search for relevant context
        match api::search(query).await {
            Ok(response) if !response.results.is_empty() => {
                let results: Vec<_> = response.results.iter().take(3).collect();

                // Build context for LLM
                let context: String = results
                    .iter()
                    .map(|r| format!("[From {}]: {}", r.document, r.content))
                    .collect::<Vec<_>>()
                    .join("\n\n");

                // Build user-friendly summary
                let context_summary: String = results
                    .iter()
                    .enumerate()
                    .map(|(i, r)| format!("{}. {} (score: {:.2})", i + 1, r.document, r.score))
                    .collect::<Vec<_>>()
                    .join("\n");

                let prompt = format!(
                    "Use the following context to answer the question. If the context doesn't contain relevant information, answer based on your knowledge.\n\n\
                    Context:\n{}\n\n\
                    Question: {}\n\n\
                    Answer:",
                    context,
                    query
                );

                (prompt, Some(context_summary))
            }
            _ => (query.to_string(), None),
        }
    }

    let send_message = move |_evt: Event<MouseData>| {
        let user_input = input_text().trim().to_string();
        if user_input.is_empty() || is_loading() {
            return;
        }

        messages.write().push(ChatMessage {
            role: "user".to_string(),
            content: user_input.clone(),
            context: None,
        });

        input_text.set(String::new());
        is_loading.set(true);
        error_msg.set(None);

        let model = selected_model();
        let use_rag = rag_enabled();

        spawn(async move {
            // Build prompt with RAG context if enabled
            let (prompt, context_summary) = build_rag_prompt(&user_input, use_rag).await;

            let request = OllamaRequest {
                model,
                prompt,
                stream: false,
            };

            match gloo_net::http::Request::post(OLLAMA_URL)
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&request).unwrap())
                .unwrap()
                .send()
                .await
            {
                Ok(response) => {
                    if response.ok() {
                        match response.json::<OllamaResponse>().await {
                            Ok(data) => {
                                messages.write().push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: data.response,
                                    context: context_summary,
                                });
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Failed to parse response: {}", e)));
                            }
                        }
                    } else {
                        error_msg.set(Some(format!("HTTP error: {}", response.status())));
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Request failed: {}. Is Ollama running?", e)));
                }
            }

            is_loading.set(false);
        });
    };

    let on_keypress = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter && !evt.modifiers().shift() {
            evt.prevent_default();
            let user_input = input_text().trim().to_string();
            if user_input.is_empty() || is_loading() {
                return;
            }

            messages.write().push(ChatMessage {
                role: "user".to_string(),
                content: user_input.clone(),
                context: None,
            });

            input_text.set(String::new());
            is_loading.set(true);
            error_msg.set(None);

            let model = selected_model();
            let use_rag = rag_enabled();

            spawn(async move {
                let (prompt, context_summary) = build_rag_prompt(&user_input, use_rag).await;

                let request = OllamaRequest {
                    model,
                    prompt,
                    stream: false,
                };

                match gloo_net::http::Request::post(OLLAMA_URL)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(response) => {
                        if response.ok() {
                            match response.json::<OllamaResponse>().await {
                                Ok(data) => {
                                    messages.write().push(ChatMessage {
                                        role: "assistant".to_string(),
                                        content: data.response,
                                        context: context_summary,
                                    });
                                }
                                Err(e) => {
                                    error_msg.set(Some(format!("Failed to parse response: {}", e)));
                                }
                            }
                        } else {
                            error_msg.set(Some(format!("HTTP error: {}", response.status())));
                        }
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Request failed: {}. Is Ollama running?", e)));
                    }
                }

                is_loading.set(false);
            });
        }
    };

    let clear_chat = move |_evt: Event<MouseData>| {
        messages.write().clear();
        error_msg.set(None);
    };

    let toggle_upload_panel = move |_evt: Event<MouseData>| {
        show_upload_panel.set(!show_upload_panel());
    };

    let refresh_documents = move |_evt: Event<MouseData>| {
        spawn(async move {
            match api::list_documents().await {
                Ok(resp) => documents.set(resp.documents),
                Err(e) => upload_status.set(Some(format!("Failed to load: {}", e))),
            }
        });
    };

    rsx! {
        // Fixed viewport container - NO scroll on body
        div {
            class: "fixed inset-0 flex bg-base-200 overflow-hidden",
            "data-theme": "dark",

            // Left sidebar - Document Upload Panel (collapsible)
            if show_upload_panel() {
                div {
                    class: "w-64 lg:w-72 bg-base-100 border-r border-base-300 flex flex-col flex-shrink-0 h-full",

                    // Panel header
                    div {
                        class: "p-2 border-b border-base-300 flex justify-between items-center flex-shrink-0",
                        h2 {
                            class: "font-bold text-sm",
                            "📁 Documents"
                        }
                        button {
                            class: "btn btn-ghost btn-xs",
                            onclick: toggle_upload_panel,
                            "✕"
                        }
                    }

                    // Upload area
                    div {
                        class: "p-2 border-b border-base-300 flex-shrink-0",

                        // File input
                        div {
                            class: "mb-1",
                            label {
                                class: "block text-xs text-base-content/70 mb-1",
                                "Upload .txt, .md, or .pdf"
                            }
                            input {
                                r#type: "file",
                                class: "file-input file-input-bordered file-input-xs w-full",
                                accept: ".txt,.md,.pdf",
                                disabled: is_uploading(),
                                onchange: move |_evt| {
                                    spawn(async move {
                                        is_uploading.set(true);
                                        upload_status.set(Some("Uploading...".to_string()));

                                        // Get file from event using web_sys
                                        let window = web_sys::window().unwrap();
                                        let document = window.document().unwrap();
                                        let input: web_sys::HtmlInputElement = document
                                            .query_selector("input[type='file']")
                                            .unwrap()
                                            .unwrap()
                                            .dyn_into()
                                            .unwrap();

                                        if let Some(files) = input.files() {
                                            if let Some(file) = files.get(0) {
                                                let filename = file.name();

                                                // Read file content
                                                let array_buffer = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                                                    .await
                                                    .unwrap();
                                                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                                                let data = uint8_array.to_vec();

                                                match api::upload_document(&filename, &data).await {
                                                    Ok(_resp) => {
                                                        upload_status.set(Some(format!("✓ {}", filename)));
                                                        // Refresh document list
                                                        if let Ok(docs) = api::list_documents().await {
                                                            documents.set(docs.documents);
                                                        }
                                                    }
                                                    Err(e) => {
                                                        upload_status.set(Some(format!("✗ {}", e)));
                                                    }
                                                }
                                            }
                                        }

                                        is_uploading.set(false);
                                    });
                                }
                            }
                        }

                        // Upload status
                        if let Some(status) = upload_status() {
                            div {
                                class: "text-xs truncate",
                                class: if status.starts_with("✓") { "text-success" } else if status.starts_with("✗") { "text-error" } else { "text-info" },
                                "{status}"
                            }
                        }
                    }

                    // Document list - scrollable
                    div {
                        class: "flex-1 overflow-y-auto p-2 min-h-0",

                        div {
                            class: "flex justify-between items-center mb-1",
                            span {
                                class: "text-xs text-base-content/70",
                                "{documents().len()} docs"
                            }
                            button {
                                class: "btn btn-ghost btn-xs",
                                onclick: refresh_documents,
                                "🔄"
                            }
                        }

                        if documents().is_empty() {
                            div {
                                class: "text-center text-xs text-base-content/50 py-2",
                                "No documents yet"
                            }
                        }

                        for doc in documents() {
                            div {
                                key: "{doc}",
                                class: "flex items-center justify-between p-1.5 bg-base-200 rounded mb-1 text-xs",
                                span {
                                    class: "truncate flex-1 mr-1",
                                    title: "{doc}",
                                    "📄 {doc}"
                                }
                                button {
                                    class: "btn btn-ghost btn-xs text-error p-0 min-h-0 h-auto",
                                    onclick: {
                                        let doc_name = doc.clone();
                                        move |_| {
                                            let doc_to_delete = doc_name.clone();
                                            spawn(async move {
                                                if let Ok(_) = api::delete_document(&doc_to_delete).await {
                                                    if let Ok(docs) = api::list_documents().await {
                                                        documents.set(docs.documents);
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "🗑"
                                }
                            }
                        }
                    }

                    // Reindex button
                    div {
                        class: "p-2 border-t border-base-300 flex-shrink-0",
                        button {
                            class: "btn btn-xs w-full",
                            style: "background-color:#0D98BA; color:white;",
                            onclick: move |_| {
                                spawn(async move {
                                    upload_status.set(Some("Reindexing...".to_string()));
                                    match api::reindex().await {
                                        Ok(_) => upload_status.set(Some("✓ Reindex done".to_string())),
                                        Err(e) => upload_status.set(Some(format!("✗ {}", e))),
                                    }
                                });
                            },
                            "🔄 Reindex"
                        }
                    }
                }
            }

            // Main chat area
            div {
                class: "flex-1 flex flex-col min-w-0 h-full",

                // Header - compact
                div {
                    class: "bg-base-100 shadow-lg flex-shrink-0 px-2 py-1.5 flex items-center justify-between",

                    div {}

                    div {
                        class: "flex items-center gap-1 sm:gap-2",

                        // Clear button
                        button {
                            class: "btn btn-ghost btn-xs",
                            onclick: clear_chat,
                            "🗑️"
                        }
                    }
                }

                // Chat container - fills remaining space, internal scroll only
                div {
                    class: "flex-1 overflow-y-auto p-2 sm:p-3 space-y-2 min-h-0",

                    // Welcome message if no messages
                    if messages().is_empty() {
                        div {
                            class: "flex items-center justify-center h-full",
                            div {
                                class: "text-center px-4",
                                h2 {
                                    class: "text-lg sm:text-xl lg:text-2xl font-bold mb-2",
                                    "Welcome to RAG Chat"
                                }
                                p {
                                    class: "text-base-content/70 mb-2 text-xs sm:text-sm",
                                    if rag_enabled() {
                                        "Ask questions using your documents + Ollama."
                                    } else {
                                        "Ask questions using Ollama."
                                    }
                                }

                                // Document count indicator
                                if !documents().is_empty() && rag_enabled() {
                                    div {
                                        class: "mb-2 text-xs sm:text-sm",
                                        style: "color:#0D98BA;",
                                        "📚 {documents().len()} documents"
                                    }
                                }

                                div {
                                    class: "flex flex-wrap gap-1 sm:gap-2 justify-center",
                                    button {
                                        class: "btn btn-outline btn-xs",
                                        onclick: move |_: Event<MouseData>| input_text.set("Explain quantum computing".to_string()),
                                        "💡 Quantum"
                                    }
                                    button {
                                        class: "btn btn-outline btn-xs",
                                        onclick: move |_: Event<MouseData>| input_text.set("Write a Rust function to sort a vector".to_string()),
                                        "🦀 Rust"
                                    }
                                    button {
                                        class: "btn btn-outline btn-xs",
                                        onclick: move |_: Event<MouseData>| input_text.set("What is RAG in AI?".to_string()),
                                        "🤖 RAG"
                                    }
                                }
                            }
                        }
                    }

                    // Messages
                    for (idx, msg) in messages().iter().enumerate() {
                        div {
                            key: "{idx}",
                            class: if msg.role == "user" { "chat chat-end" } else { "chat chat-start" },

                            div {
                                class: "chat-image avatar placeholder",
                                div {
                                    class: if msg.role == "user" {
                                        "bg-[#0D98BA] text-white rounded-full w-6 sm:w-8"
                                    } else {
                                        "bg-cyan-500 text-white rounded-full w-6 sm:w-8"
                                    },
                                    span {
                                        class: "text-xs sm:text-sm",
                                        if msg.role == "user" { "👤" } else { "🤖" }
                                    }
                                }
                            }

                            div {
                                class: if msg.role == "user" {
                                    "chat-bubble text-xs sm:text-sm bg-[#0D98BA] text-white max-w-[85%] sm:max-w-[75%]"
                                } else {
                                    "chat-bubble text-xs sm:text-sm bg-cyan-600 text-white max-w-[85%] sm:max-w-[75%]"
                                },
                                pre {
                                    class: "whitespace-pre-wrap font-sans",
                                    "{msg.content}"
                                }

                                // Show RAG context if available (for assistant messages)
                                if msg.role == "assistant" {
                                    if let Some(ctx) = &msg.context {
                                        div {
                                            class: "mt-2 pt-2 border-t border-white/20 text-xs opacity-70",
                                            div {
                                                class: "font-semibold mb-1",
                                                "📚 Sources used:"
                                            }
                                            pre {
                                                class: "whitespace-pre-wrap font-sans",
                                                "{ctx}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Loading indicator
                    if is_loading() {
                        div {
                            class: "chat chat-start",

                            div {
                                class: "chat-image avatar placeholder",
                                div {
                                    class: "bg-teal-500 text-white rounded-full w-6 sm:w-8",
                                    span { class: "text-xs sm:text-sm", "🤖" }
                                }
                            }

                            div {
                                class: "chat-bubble bg-cyan-600 text-white",
                                span {
                                    class: "loading loading-dots loading-xs sm:loading-sm text-white"
                                }
                            }
                        }
                    }

                    // Info/Error message
                    if let Some(msg) = error_msg() {
                        div {
                            class: if msg.starts_with("[INFO]") {
                                "alert alert-info shadow-lg text-sm py-2 cursor-pointer"
                            } else {
                                "alert alert-error shadow-lg text-xs py-1"
                            },
                            onclick: move |_| error_msg.set(None),
                            pre {
                                class: "whitespace-pre-wrap font-sans",
                                "{msg}"
                            }
                        }
                    }
                }

                // Input area - compact, fixed at bottom
                div {
                    class: "p-2 sm:p-3 bg-base-100 border-t border-base-300 flex-shrink-0",

                    div {
                        class: "max-w-4xl mx-auto",

                        div {
                            class: "flex gap-2 items-center",

                            // Left buttons: RAG toggle and Info
                            div {
                                class: "flex flex-col gap-2 items-center",

                                // RAG toggle switch
                                div {
                                    class: "flex flex-col items-center gap-0.5",
                                    label {
                                        class: "text-sm font-medium",
                                        style: "color: white;",
                                        "RAG"
                                    }
                                    input {
                                        r#type: "checkbox",
                                        class: "toggle toggle-md",
                                        style: if rag_enabled() { "background-color:#0D98BA; border-color:#0D98BA;" } else { "" },
                                        checked: rag_enabled(),
                                        onchange: move |_| rag_enabled.set(!rag_enabled()),
                                        title: if rag_enabled() { "RAG is ON - Click to disable" } else { "RAG is OFF - Click to enable" },
                                    }
                                }

                                // Info button
                                button {
                                    class: "btn btn-sm rounded-full px-4 text-sm font-medium",
                                    style: "border: 1.5px solid rgba(255,255,255,0.3); background: transparent; color: white; min-height: 1.5rem; height: 1.5rem;",
                                    onclick: move |_| show_info.set(!show_info()),
                                    title: "Show RAG info",
                                    "Info"
                                }

                                // Add documents button
                                button {
                                    class: "btn btn-sm rounded-full px-4 text-lg font-bold",
                                    style: "border: 1.5px solid rgba(255,255,255,0.3); background: transparent; color: white; min-height: 1.5rem; height: 1.5rem;",
                                    onclick: move |_| show_upload_panel.set(!show_upload_panel()),
                                    title: "Toggle documents panel",
                                    "+"
                                }
                            }

                            input {
                                class: "input input-bordered flex-1 text-sm sm:text-base px-3 sm:px-4 rounded-full focus:border-[#0D98BA] focus:outline-none focus:ring-1 focus:ring-[#0D98BA]",
                                style: "height: 5rem;",
                                r#type: "text",
                                placeholder: "Type a message...",
                                value: "{input_text}",
                                oninput: move |evt| input_text.set(evt.value()),
                                onkeypress: on_keypress,
                                disabled: is_loading(),
                            }

                            button {
                                class: "btn btn-lg rounded-3xl px-10 text-xl text-white hover:opacity-90",
                                style: "border-radius:32px; background-color:#0D98BA;",
                                onclick: send_message,
                                disabled: is_loading() || input_text().trim().is_empty(),

                                if is_loading() {
                                    span { class: "loading loading-spinner loading-xs sm:loading-sm" }
                                } else {
                                    "Send"
                                }
                            }
                        }

                        // Status bar - minimal
                        div {
                            class: "flex justify-between items-center mt-1 text-xs text-base-content/50 px-1",

                            span {
                                class: "truncate",
                                if rag_enabled() {
                                    "{selected_model} • RAG"
                                } else {
                                    "{selected_model}"
                                }
                            }

                            span {
                                "{messages().len()} msgs"
                            }
                        }
                    }
                }
            }

            // Info Panel Overlay
            if show_info() {
                div {
                    class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                    onclick: move |_| show_info.set(false),

                    div {
                        class: "bg-base-100 rounded-2xl p-5 sm:p-6 max-w-7xl w-full mx-2 sm:mx-4 shadow-2xl max-h-[90vh] flex flex-col gap-4",
                        onclick: move |evt| evt.stop_propagation(),

                        div {
                            class: "flex-1 overflow-y-auto space-y-2.5 pr-0 sm:pr-1",

                            h3 {
                                class: "text-lg font-bold leading-tight",
                                style: "color: #0D98BA;",
                                if rag_enabled() { "📚 RAG is ON" } else { "💬 RAG is OFF" }
                            }

                            div {
                                class: "grid gap-3.5 lg:gap-5 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] text-sm",

                                // Column: RAG behavior + current config snapshot
                                div {
                                    class: "space-y-2",
                                    if rag_enabled() {
                                        p { "• Your question is searched against ", strong { "{documents().len()}" }, " uploaded documents" }
                                        p { "• The LLM answers using both context + its knowledge" }
                                        p { "• Top 3 relevant chunks are added as context" }
                                    } else {
                                        p { "• Questions go directly to Ollama" }
                                        p { "• No document context is used" }
                                        p { "• Toggle RAG ON to use your ", strong { "{documents().len()}" }, " documents" }
                                    }

                                    div {
                                        class: "pt-1.5 border-t border-base-300",
                                        p {
                                            class: "leading-tight mb-3",
                                            "Chunk size is the length of each document slice we embed. Bigger chunks keep more context per slice, while smaller ones stay precise but risk missing surrounding details. We aim for enough tokens to capture a full thought without blowing the model's context window."
                                        }
                                        p {
                                            class: "text-xs font-medium mb-2 mt-1",
                                            style: "color: #0D98BA;",
                                            "Current Chunk Settings"
                                        }
                                        table {
                                            class: "table table-xs w-full text-[0.72rem] leading-tight",
                                            tbody {
                                                tr {
                                                    td { class: "text-xs", "target_size" }
                                                    td { class: "text-xs font-mono", "384" }
                                                    td { class: "text-xs text-base-content/60", "Target tokens" }
                                                }
                                                tr {
                                                    td { class: "text-xs", "min_size" }
                                                    td { class: "text-xs font-mono", "192" }
                                                    td { class: "text-xs text-base-content/60", "Minimum tokens" }
                                                }
                                                tr {
                                                    td { class: "text-xs", "max_size" }
                                                    td { class: "text-xs font-mono", "512" }
                                                    td { class: "text-xs text-base-content/60", "Maximum tokens" }
                                                }
                                                tr {
                                                    td { class: "text-xs", "overlap" }
                                                    td { class: "text-xs font-mono", "50" }
                                                    td { class: "text-xs text-base-content/60", "Overlap tokens" }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Column: Chunk guidance reference
                                div {
                                        class: "space-y-2",

                                        div {
                                            class: "border border-base-300 rounded-xl p-2.5 sm:p-3.5",
                                            h4 {
                                                class: "font-semibold text-sm mb-3",
                                                "How to Decide on Chunk Size"
                                            }

                                            // 1. Model Context Window
                                            div {
                                                class: "mb-1.5",
                                                p { class: "text-xs font-medium mb-1", "1. Model Context Window" }
                                                table {
                                                    class: "table table-xs w-full text-[0.72rem] leading-tight",
                                                    thead {
                                                        tr {
                                                            th { class: "text-xs", "Model" }
                                                            th { class: "text-xs", "Context" }
                                                            th { class: "text-xs", "Recommended" }
                                                        }
                                                    }
                                                    tbody {
                                                        tr {
                                                            td { class: "text-xs", "phi (2K)" }
                                                            td { class: "text-xs", "2,048" }
                                                            td { class: "text-xs", "256-384" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "llama2 (4K)" }
                                                            td { class: "text-xs", "4,096" }
                                                            td { class: "text-xs", "384-512" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "llama3 (8K)" }
                                                            td { class: "text-xs", "8,192" }
                                                            td { class: "text-xs", "512-768" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "GPT-3.5 (16K)" }
                                                            td { class: "text-xs", "16,384" }
                                                            td { class: "text-xs", "768-1024" }
                                                        }
                                                    }
                                                }
                                                p {
                                                    class: "text-xs text-base-content/60 mt-0.5 italic leading-tight",
                                                    "Rule: chunk_size × num_chunks + question + answer < context_window"
                                                }
                                            }

                                            // 2. Content Type
                                            div {
                                                class: "mb-2",
                                                p { class: "text-xs font-medium mb-1", "2. Content Type" }
                                                table {
                                                    class: "table table-xs w-full text-[0.72rem] leading-tight",
                                                    thead {
                                                        tr {
                                                            th { class: "text-xs", "Content" }
                                                            th { class: "text-xs", "Size" }
                                                            th { class: "text-xs", "Why" }
                                                        }
                                                    }
                                                    tbody {
                                                        tr {
                                                            td { class: "text-xs", "Technical docs" }
                                                            td { class: "text-xs", "512-768" }
                                                            td { class: "text-xs", "Concepts need context" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "Code" }
                                                            td { class: "text-xs", "256-512" }
                                                            td { class: "text-xs", "Functions self-contained" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "Legal/contracts" }
                                                            td { class: "text-xs", "768-1024" }
                                                            td { class: "text-xs", "Clauses need full context" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "Chat logs" }
                                                            td { class: "text-xs", "256-384" }
                                                            td { class: "text-xs", "Short exchanges" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "Academic papers" }
                                                            td { class: "text-xs", "512-768" }
                                                            td { class: "text-xs", "Paragraphs meaningful" }
                                                        }
                                                    }
                                                }
                                            }

                                            // 3. Retrieval Quality Trade-offs
                                            div {
                                                class: "mb-2",
                                                p { class: "text-xs font-medium mb-1", "3. Retrieval Quality Trade-offs" }
                                                table {
                                                    class: "table table-xs w-full text-[0.72rem] leading-tight",
                                                    thead {
                                                        tr {
                                                            th { class: "text-xs", "Size" }
                                                            th { class: "text-xs", "Pros" }
                                                            th { class: "text-xs", "Cons" }
                                                        }
                                                    }
                                                    tbody {
                                                        tr {
                                                            td { class: "text-xs", "Small (256)" }
                                                            td { class: "text-xs", "Precise, granular" }
                                                            td { class: "text-xs", "May lose context" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "Medium (512)" }
                                                            td { class: "text-xs", "Good balance" }
                                                            td { class: "text-xs", "Standard choice" }
                                                        }
                                                        tr {
                                                            td { class: "text-xs", "Large (768+)" }
                                                            td { class: "text-xs", "More context" }
                                                            td { class: "text-xs", "Less precise" }
                                                        }
                                                    }
                                                }
                                            }

                                            // 4. Overlap Considerations
                                            div {
                                                class: "mb-2",
                                                p { class: "text-xs font-medium mb-1", "4. Overlap Considerations" }
                                                ul {
                                                    class: "text-xs space-y-0.5 list-disc list-inside text-base-content/80 leading-tight",
                                                    li { class: "text-xs", "0 overlap: Risk losing context at boundaries" }
                                                    li { class: "text-xs", "50-100 tokens (10-20%): Good balance" }
                                                    li { class: "text-xs", "150+ tokens: Redundant, wastes space" }
                                                }
                                            }
                                        }
                                    }
                                }

                            div {
                                class: "pt-2 border-t border-base-300 text-xs text-base-content/70",
                                "Model: {selected_model}"
                            }
                        }

                        button {
                            class: "btn btn-sm w-full mt-0.5 text-base",
                            style: "background-color: #0D98BA; color: white; font-size: 1.25rem;",
                            onclick: move |_| show_info.set(false),
                            "Got it"
                        }
                    }
                }
            }
        }
    }
}
