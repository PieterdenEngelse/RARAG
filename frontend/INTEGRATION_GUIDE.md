# Frontend-Backend Integration Guide

## ✅ Completed Setup

### Backend Changes (in `/home/pde/ag`)
1. **Port changed**: API server now runs on `http://localhost:3000` (was 8080)
2. **CORS enabled**: Frontend at `http://localhost:8080` can now make requests
3. **Dependencies added**: `actix-cors = "0.7"` in Cargo.toml

### Frontend Changes (in `/home/pde/ag/frontend/fro`)
1. **API client created**: `src/api.rs` with functions to call backend
2. **Dependencies added**:
   - `gloo-net = "0.6"` - HTTP client for WASM
   - `urlencoding = "2.1"` - URL encoding
   - `serde = "1.0"` - JSON serialization
   - `serde_json = "1.0"` - JSON handling
3. **Module registered**: API module added to `src/lib.rs`

## 🚀 How to Run

### Terminal 1: Start Backend
```bash
cd /home/pde/ag
cargo run
```
Backend will run on `http://localhost:3000`

### Terminal 2: Start Frontend
```bash
cd /home/pde/ag/frontend/fro
dx serve
```
Frontend will run on `http://localhost:8080`

## 📡 Available API Functions

In your Dioxus components, you can now use:

```rust
use crate::api;

// Check backend health
let health = api::health_check().await?;

// Search documents
let results = api::search("your query").await?;

// List all documents
let docs = api::list_documents().await?;

// Delete a document
api::delete_document("filename.pdf").await?;

// Trigger reindexing
api::reindex().await?;
```

## 🎯 Next Steps

### 1. Create a Search Component
Create `src/components/search.rs`:
```rust
use dioxus::prelude::*;
use crate::api;

#[component]
pub fn SearchBar() -> Element {
    let mut query = use_signal(|| String::new());
    let mut results = use_signal(|| Vec::new());
    let mut loading = use_signal(|| false);

    let search = move |_| {
        spawn(async move {
            loading.set(true);
            match api::search(&query()).await {
                Ok(response) => results.set(response.results),
                Err(e) => log::error!("Search failed: {}", e),
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "search-container",
            input {
                class: "search-input",
                r#type: "text",
                placeholder: "Search documents...",
                value: "{query}",
                oninput: move |e| query.set(e.value().clone()),
            }
            button {
                class: "search-button",
                onclick: search,
                disabled: loading(),
                if loading() { "Searching..." } else { "Search" }
            }
            
            div { class: "results",
                for result in results() {
                    div { class: "result-item",
                        p { "{result.content}" }
                        small { "Score: {result.score} | Document: {result.document}" }
                    }
                }
            }
        }
    }
}
```

### 2. Add Search to a Page
In `src/pages/home.rs` or wherever you want:
```rust
use crate::components::search::SearchBar;

rsx! {
    div {
        h1 { "Rust Agentic RAG" }
        SearchBar {}
    }
}
```

### 3. Create Document Upload Component
Create `src/components/upload.rs` for file uploads

### 4. Add Document List Component
Create `src/components/documents.rs` to list and manage documents

## 🔧 Troubleshooting

### CORS Errors
If you see CORS errors, ensure:
- Backend is running on port 3000
- Frontend is running on port 8080
- Both servers are started

### Connection Refused
- Make sure backend is running: `cd /home/pde/ag && cargo run`
- Check backend logs for errors

### Build Errors
- Run `cargo clean` in both frontend and backend
- Rebuild: `cargo build`

## 📚 API Endpoints Reference

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check with metrics |
| GET | `/ready` | Readiness check |
| GET | `/metrics` | System metrics |
| POST | `/upload` | Upload documents |
| GET | `/documents` | List all documents |
| DELETE | `/documents/{filename}` | Delete a document |
| POST | `/reindex` | Reindex all documents |
| GET | `/search?q=query` | Search documents |
| POST | `/rerank` | Rerank results |
| POST | `/summarize` | Summarize chunks |
| POST | `/save_vectors` | Save vectors |

## 🎨 Styling Tips

Add Tailwind classes to your components for styling:
- `class: "p-4 bg-white dark:bg-gray-800 rounded-lg shadow"`
- `class: "text-sm text-gray-600 dark:text-gray-300"`
- `class: "hover:bg-indigo-600 transition-colors"`
