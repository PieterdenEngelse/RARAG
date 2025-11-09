# Deployment & Configuration Guide

## System Requirements

**Minimum Hardware:**
- CPU: Intel Core i3 or equivalent (4 cores, 2.1 GHz minimum)
- RAM: 8GB
- Storage: 10GB SSD

**Tested On:**
- Intel Core i3-10110U @ 2.10GHz, 8GB RAM ✓

## Prerequisites

### 1. Rust Toolchain
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

### 2. Ollama (for LLM - Phi 3.5)
```bash
# Install Ollama from https://ollama.ai

# Pull Phi 3.5 model (3.8B, uses ~4GB RAM)
ollama pull phi:3.5

# Run Ollama server (keep this running)
ollama serve
```

**Alternative models:**
```bash
ollama pull mistral:7b      # Mistral 7B (5-6GB)
ollama pull qwen:7b         # Qwen 7B (5-6GB) 
ollama pull neural-chat:7b  # Intel optimized (5-6GB)
```

## Build & Run

### Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run end-to-end tests
cargo test --test e2e_test

# Run example
cargo run --example agent_example

# Run with logging
RUST_LOG=debug cargo run
```

### Production

```bash
# Build release (optimized)
cargo build --release

# Run release binary
./target/release/ag
```

## Configuration

### Environment Variables

```bash
# API Server
export BACKEND_HOST=0.0.0.0
export BACKEND_PORT=3010

# Logging
export RUST_LOG=info
# RUST_LOG=debug for detailed logs

# LLM Configuration
export LLM_MODEL=phi:3.5
export OLLAMA_URL=http://localhost:11434
```

### API Configuration

Create `.env` file in project root:
```toml
BACKEND_HOST=0.0.0.0
BACKEND_PORT=3010
RUST_LOG=info
```

## Running the System

### Step 1: Start Ollama

```bash
# Terminal 1
ollama serve
# Output should show: Listening on 127.0.0.1:11434
```

### Step 2: Start Backend

```bash
# Terminal 2
cargo run

# Output should show:
# 📦 Initializing Retriever...
# 🚀 Starting API server on http://127.0.0.1:3010 ...
```

### Step 3: Test the System

```bash
# Terminal 3

# Health check
curl http://localhost:3010/health

# Create a goal
curl -X POST http://localhost:3010/api/agent/goals \
  -H "Content-Type: application/json" \
  -d '{"goal": "Learn about Rust"}'

# Record an episode
curl -X POST http://localhost:3010/api/agent/episodes \
  -H "Content-Type: application/json" \
  -d '{
    "query": "What is Rust?",
    "response": "Rust is a systems programming language",
    "context_chunks_used": 3,
    "success": true
  }'

# Get agent context
curl http://localhost:3010/api/agent/context
```

## API Endpoints

### Core Health
- `GET /health` - System health check
- `GET /ready` - Readiness check
- `GET /metrics` - System metrics

### Memory API (Phase 4)
- `POST /api/memory/add` - Add chunk to vector store
- `POST /api/memory/batch` - Batch add chunks
- `POST /api/memory/search` - Search vectors
- `GET /api/memory/stats` - Store statistics

### Agent API (Phase 6)
- `POST /api/agent/goals` - Create goal
- `GET /api/agent/goals` - List active goals
- `PUT /api/agent/goals/{id}` - Complete goal
- `POST /api/agent/episodes` - Record episode
- `POST /api/agent/episodes/similar` - Find similar queries
- `POST /api/agent/reflections` - Trigger reflection
- `GET /api/agent/context` - Get agent context
- `GET /api/agent/health` - Agent health check

## Troubleshooting

### Issue: Connection refused to Ollama
```
Error: LLM connection failed: Cannot reach Ollama at http://localhost:11434
```
**Solution:**
- Ensure Ollama is running: `ollama serve`
- Check Ollama is accessible: `curl http://localhost:11434/api/tags`

### Issue: Out of memory
```
System runs out of memory with Phi 3.5
```
**Solution:**
- Phi 3.5 needs ~4GB RAM minimum
- Close other applications
- Use smaller model: `ollama pull phi:2.5` (2.7B)

### Issue: Slow responses
```
API responses take > 10 seconds
```
**Solution:**
- Ollama inference is CPU-bound on i3-10110U
- Phi 3.5 generates ~10-15 tokens/second on this hardware
- This is expected. Use smaller model if needed.

### Issue: Database locked
```
Error: database is locked
```
**Solution:**
```bash
# Kill any running processes
pkill -f ag

# Wait 2 seconds
sleep 2

# Restart
cargo run
```

## Monitoring

### View Logs
```bash
# Terminal with service running
RUST_LOG=debug cargo run

# Or filter logs
RUST_LOG=memory::agent=debug cargo run
```

### Check Database
```bash
# View agent memory
sqlite3 agent_memory.db ".tables"
sqlite3 agent_memory.db "SELECT * FROM goals;"
```

## Performance Metrics

**On Intel i3-10110U (4 cores @ 2.1GHz):**
- Embedding generation: ~50-100ms per chunk
- Vector search (1000 vectors): ~10-20ms
- LLM inference (Phi 3.5): ~10-15 tokens/second
- API response time: 100-500ms (depending on LLM)

## Scaling to Production

For production deployment:

1. **Use Docker:**
```dockerfile
FROM rust:latest
WORKDIR /app
COPY . .
RUN cargo build --release
EXPOSE 3010
CMD ["./target/release/ag"]
```

2. **Enable SSL/TLS** - Use nginx reverse proxy

3. **Database** - Consider PostgreSQL for agent_memory

4. **LLM** - Use Ollama server or managed service

5. **Monitoring** - Add Prometheus metrics

6. **Load Balancing** - Multiple instances with shared database

## Next Steps

1. **Phase 8: Multi-Agent Collaboration** - Multiple agents coordinating
2. **Advanced LLM Integration** - Streaming responses, fine-tuning
3. **Database Migration** - PostgreSQL for scalability
4. **Distributed Caching** - Redis for shared embeddings
5. **Observability** - Tracing and metrics collection