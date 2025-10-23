# Quick Start Guide - Search Integration Demo

## ✅ What Was Created

### 1. Search Component (`src/components/search.rs`)
A fully functional search interface with:
- Real-time backend health check
- Search input with Enter key support
- Loading states and error handling
- Beautiful results display with scores
- Responsive Tailwind styling
- Dark mode support

### 2. Updated Home Page (`src/pages/home.rs`)
- Clean, modern landing page
- Integrated SearchBar component
- Info cards explaining the RAG system
- Full-width responsive layout

## 🚀 How to Test

### Step 1: Start the Backend
```bash
cd /home/pde/ag
cargo run
```

You should see:
```
📦 Initializing Retriever...
🚀 Starting API server...
```

Backend runs on: **http://localhost:8081**

### Step 2: Start the Frontend
```bash
cd /home/pde/ag/frontend/fro
dx serve
```

Frontend runs on: **http://localhost:8080**

### Step 3: Add Some Documents

**Option A: Via API (using curl)**
```bash
# Upload a text file
curl -X POST http://localhost:8081/upload \
  -F "file=@/path/to/your/document.txt"

# Reindex
curl -X POST http://localhost:8081/reindex
```

**Option B: Manually**
```bash
# Copy files to documents directory
cp your-file.txt /home/pde/ag/documents/
cp your-file.pdf /home/pde/ag/documents/

# Restart backend (it auto-indexes on startup)
```

### Step 4: Search!

1. Open http://localhost:8080 in your browser
2. You should see:
   - "Backend: ✓ Connected (X docs, Y vectors)" at the top
   - A search bar
   - Three info cards below
3. Type a query and press Enter or click Search
4. Results will appear with:
   - Relevance score
   - Document name
   - Matching content snippet

## 🎨 Features Demonstrated

### Backend Connection
- Health check on component mount
- Real-time status display
- Error handling for offline backend

### Search Functionality
- Async API calls using `gloo-net`
- Loading states
- Error messages
- Empty state handling
- Keyboard shortcuts (Enter to search)

### UI/UX
- Tailwind CSS styling
- Dark mode support
- Responsive design
- Smooth transitions
- Loading spinner
- Result cards with hover effects

## 🔧 Troubleshooting

### "Backend offline" message
- Make sure backend is running on port 8081
- Check: `curl http://localhost:8081/health`

### No results found
- Upload documents to `/home/pde/ag/documents/`
- Trigger reindex: `curl -X POST http://localhost:8081/reindex`
- Check backend logs for indexing status

### CORS errors
- Backend CORS is configured for `http://localhost:8080`
- Make sure frontend is on port 8080
- Check browser console for specific errors

### Build errors
```bash
# Clean and rebuild frontend
cd /home/pde/ag/frontend/fro
cargo clean
dx serve

# Clean and rebuild backend
cd /home/pde/ag
cargo clean
cargo run
```

## 📝 Sample Test Documents

Create a test file to search:

```bash
cat > /home/pde/ag/documents/rust-intro.txt << 'EOF'
Rust is a systems programming language that runs blazingly fast, prevents segfaults, and guarantees thread safety. It accomplishes these goals without needing a garbage collector, making it a useful language for a number of use cases other languages aren't good at: embedding in other languages, programs with specific space and time requirements, and writing low-level code, like device drivers and operating systems.
EOF

cat > /home/pde/ag/documents/ai-ml.txt << 'EOF'
Machine learning is a subset of artificial intelligence that provides systems the ability to automatically learn and improve from experience without being explicitly programmed. Machine learning focuses on the development of computer programs that can access data and use it to learn for themselves.
EOF
```

Then restart the backend or call reindex:
```bash
curl -X POST http://localhost:8081/reindex
```

Now search for "Rust programming" or "machine learning"!

## 🎯 Next Steps

1. **Add Document Upload UI**
   - Create upload component
   - File picker
   - Progress indicator

2. **Add Document Management**
   - List all documents
   - Delete documents
   - View document details

3. **Enhance Search**
   - Filters (by document, date, etc.)
   - Sort options
   - Pagination for many results

4. **Add More Features**
   - Summarization UI
   - Reranking controls
   - Export results
   - Save searches

## 📚 Code Reference

### Using the API in Other Components

```rust
use crate::api;

// In your component
let search_results = use_signal(|| Vec::new());

spawn(async move {
    match api::search("your query").await {
        Ok(response) => search_results.set(response.results),
        Err(e) => log::error!("Search failed: {}", e),
    }
});
```

### Available API Functions

- `api::health_check()` - Backend health
- `api::search(query)` - Search documents
- `api::list_documents()` - List all docs
- `api::delete_document(filename)` - Delete a doc
- `api::reindex()` - Trigger reindex

All functions return `Result<T, String>` and are async.
