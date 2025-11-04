# Agentic RAG Installation Guide
**Version**: 1.0.0  
**Date**: 2025-11-03  
**Status**: Complete & Ready for Deployment  
**Platforms**: Windows, Linux, macOS

---

## 📋 TABLE OF CONTENTS

1. [Overview](#overview)
2. [System Requirements](#system-requirements)
3. [Pre-Installation Checklist](#pre-installation-checklist)
4. [Installation Steps](#installation-steps)
5. [Platform-Specific Guides](#platform-specific-guides)
6. [Configuration](#configuration)
7. [Post-Installation Verification](#post-installation-verification)
8. [Monitoring & Health Checks](#monitoring--health-checks)
9. [Troubleshooting](#troubleshooting)
10. [Uninstallation](#uninstallation)
11. [Installer Integration Points](#installer-integration-points)

---

## OVERVIEW

The Agentic RAG installer sets up a production-ready Retrieval-Augmented Generation system with:
- Rust backend (Actix web framework)
- Dioxus frontend (WebAssembly)
- Vector search and embedding capabilities
- Comprehensive monitoring (tracing, metrics, health checks)
- Cross-platform support (Windows, Linux, macOS)

**Total Installation Time**: 15-30 minutes (varies by internet speed)

---

## SYSTEM REQUIREMENTS

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **CPU** | 2+ cores |
| **RAM** | 4 GB |
| **Disk** | 5 GB available |
| **OS** | Windows 10+, Ubuntu 20.04+, macOS 10.15+ |

### Recommended Requirements

| Component | Specification |
|-----------|----------------|
| **CPU** | 4+ cores |
| **RAM** | 8+ GB |
| **Disk** | 20+ GB SSD |
| **Network** | 10+ Mbps (for initial download) |

### Required Software

#### All Platforms
- **Rust**: 1.70+ (install from https://rustup.rs)
- **Cargo**: Included with Rust
- **Git**: Latest version

#### Windows
- **Visual Studio Build Tools 2019+** OR **Visual Studio Community 2019+**
- **Windows Terminal** (optional but recommended)

#### Linux
- **build-essential** (Debian/Ubuntu): `sudo apt-get install build-essential`
- **pkg-config**: `sudo apt-get install pkg-config`
- **OpenSSL dev libraries**: `sudo apt-get install libssl-dev`

#### macOS
- **Xcode Command Line Tools**: `xcode-select --install`
- **Homebrew** (optional): https://brew.sh

---

## PRE-INSTALLATION CHECKLIST

Before starting installation:

```
System Verification
  ☐ OS is Windows 10+, Ubuntu 20.04+, or macOS 10.15+
  ☐ At least 5 GB free disk space
  ☐ Internet connection available (500 MB+ bandwidth)
  ☐ Administrator/sudo access available

Software Verification
  ☐ Rust installed: rustc --version
  ☐ Cargo installed: cargo --version
  ☐ Git installed: git --version
  ☐ Build tools installed (platform-specific)

Environment Verification
  ☐ No conflicting applications on port 3000
  ☐ No conflicting applications on port 8080
  ☐ Home directory is writable
  ☐ Firewall allows localhost connections

Project Files Verification
  ☐ Source code cloned/extracted
  ☐ Cargo.toml exists in project root
  ☐ All dependencies listed in Cargo.toml
```

---

## INSTALLATION STEPS

### Step 1: Verify Rust Installation (2 minutes)

**Windows (PowerShell):**
```powershell
rustc --version
cargo --version
rustup update
```

**Linux/macOS (Terminal):**
```bash
rustc --version
cargo --version
rustup update
```

Expected output:
```
rustc 1.70.0 (90c541806 2023-05-31)
cargo 1.70.0 (ec8d8defa 2023-04-25)
```

**If Rust is not installed**, visit https://rustup.rs and follow instructions.

### Step 2: Clone/Extract Project (2 minutes)

**Using Git (Recommended):**
```bash
git clone https://github.com/yourusername/agentic-rag.git
cd agentic-rag
```

**Manual Extract:**
- Extract provided ZIP/TAR file
- Navigate to project directory

### Step 3: Create Configuration Directories (1 minute)

**Windows (PowerShell):**
```powershell
# Create main config directory
$configDir = "$env:USERPROFILE\.agentic-rag"
New-Item -ItemType Directory -Force -Path $configDir | Out-Null

# Create subdirectories
New-Item -ItemType Directory -Force -Path "$configDir\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$configDir\data" | Out-Null
New-Item -ItemType Directory -Force -Path "$configDir\uploads" | Out-Null
New-Item -ItemType Directory -Force -Path "$configDir\models" | Out-Null

Write-Host "✓ Configuration directories created"
```

**Linux/macOS (Bash):**
```bash
# Create main config directory
mkdir -p ~/.agentic-rag/{logs,data,uploads,models}
chmod 755 ~/.agentic-rag

echo "✓ Configuration directories created"
```

### Step 4: Set Environment Variables (2 minutes)

**Windows (PowerShell - Permanent):**
```powershell
# Set user environment variables
[Environment]::SetEnvironmentVariable("RUST_LOG", "info", "User")
[Environment]::SetEnvironmentVariable("MONITORING_ENABLED", "true", "User")
[Environment]::SetEnvironmentVariable("LOG_RETENTION_DAYS", "7", "User")
[Environment]::SetEnvironmentVariable("LOG_FORMAT", "json", "User")
[Environment]::SetEnvironmentVariable("AGENTIC_RAG_HOME", "$env:USERPROFILE\.agentic-rag", "User")

# Refresh environment
$env:RUST_LOG = "info"
$env:MONITORING_ENABLED = "true"
$env:LOG_RETENTION_DAYS = "7"
$env:LOG_FORMAT = "json"
$env:AGENTIC_RAG_HOME = "$env:USERPROFILE\.agentic-rag"

Write-Host "✓ Environment variables set"
```

**Linux/macOS (Bash):**

Add to `~/.bashrc` or `~/.zshrc`:
```bash
# Agentic RAG Configuration
export RUST_LOG=info
export MONITORING_ENABLED=true
export LOG_RETENTION_DAYS=7
export LOG_FORMAT=json
export AGENTIC_RAG_HOME=$HOME/.agentic-rag
```

Then reload:
```bash
source ~/.bashrc  # or ~/.zshrc for macOS
echo "✓ Environment variables set"
```

### Step 5: Verify Dependencies (3 minutes)

**Windows (PowerShell):**
```powershell
cd project-root
cargo check
```

**Linux/macOS (Bash):**
```bash
cd project-root
cargo check
```

Expected output ends with:
```
   Compiling agentic_rag v0.1.0
    Finished dev [unoptimized + debuginfo] target(s) in 5.23s
```

### Step 6: Build Project (5-15 minutes, depends on system)

**Windows (PowerShell):**
```powershell
cargo build --release
```

**Linux/macOS (Bash):**
```bash
cargo build --release
```

Monitor progress - this downloads and compiles all dependencies.

Expected final output:
```
    Finished release [optimized] target(s) in 2m 34s
```

### Step 7: Verify Build Artifacts (1 minute)

**Windows (PowerShell):**
```powershell
ls target/release/agentic_rag.exe  # Backend binary
ls target/release/fro.wasm         # Frontend binary (if available)
Write-Host "✓ Build artifacts verified"
```

**Linux/macOS (Bash):**
```bash
ls -lh target/release/agentic_rag   # Backend binary
ls -lh target/release/fro.wasm      # Frontend binary (if available)
echo "✓ Build artifacts verified"
```

### Step 8: Create Initial Configuration (2 minutes)

**Create `~/.agentic-rag/config.toml`:**

```toml
# Agentic RAG Configuration
# Version: 1.0.0

[server]
host = "127.0.0.1"
port = 3000
worker_threads = 4

[monitoring]
enabled = true
log_level = "info"
log_format = "json"
metrics_enabled = true
health_check_interval_secs = 30

[logging]
file_path = "~/.agentic-rag/logs"
max_log_files = 7
rotation = "daily"

[storage]
data_path = "~/.agentic-rag/data"
uploads_path = "~/.agentic-rag/uploads"
models_path = "~/.agentic-rag/models"

[features]
rag_enabled = true
monitoring_enabled = true
health_checks_enabled = true
```

---

## PLATFORM-SPECIFIC GUIDES

### Windows Installation

#### Prerequisites
```powershell
# Verify Rust
rustc --version
cargo --version

# Verify build tools
& "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe" -all
```

#### Installation Script (PowerShell)

```powershell
# agentic-rag-install.ps1
# Run as Administrator

param(
    [string]$ProjectPath = (Get-Location),
    [string]$InstallMode = "release"  # or "debug"
)

Write-Host "🚀 Agentic RAG Installer for Windows" -ForegroundColor Cyan
Write-Host "Version: 1.0.0`n" -ForegroundColor Cyan

# Step 1: Verify Prerequisites
Write-Host "📋 Verifying prerequisites..." -ForegroundColor Yellow
try {
    $rustVersion = rustc --version
    $cargoVersion = cargo --version
    Write-Host "✓ Rust: $rustVersion"
    Write-Host "✓ Cargo: $cargoVersion"
} catch {
    Write-Host "✗ Rust not found. Please install from https://rustup.rs" -ForegroundColor Red
    exit 1
}

# Step 2: Create Directories
Write-Host "`n📁 Creating configuration directories..." -ForegroundColor Yellow
$configDir = "$env:USERPROFILE\.agentic-rag"
New-Item -ItemType Directory -Force -Path "$configDir\logs" | Out-Null
New-Item -ItemType Directory -Force -Path "$configDir\data" | Out-Null
New-Item -ItemType Directory -Force -Path "$configDir\uploads" | Out-Null
New-Item -ItemType Directory -Force -Path "$configDir\models" | Out-Null
Write-Host "✓ Created: $configDir"

# Step 3: Set Environment Variables
Write-Host "`n🔧 Setting environment variables..." -ForegroundColor Yellow
[Environment]::SetEnvironmentVariable("RUST_LOG", "info", "User")
[Environment]::SetEnvironmentVariable("MONITORING_ENABLED", "true", "User")
[Environment]::SetEnvironmentVariable("LOG_RETENTION_DAYS", "7", "User")
[Environment]::SetEnvironmentVariable("LOG_FORMAT", "json", "User")
[Environment]::SetEnvironmentVariable("AGENTIC_RAG_HOME", $configDir, "User")
Write-Host "✓ Environment variables configured"

# Step 4: Update Rust
Write-Host "`n📦 Updating Rust toolchain..." -ForegroundColor Yellow
rustup update

# Step 5: Build Project
Write-Host "`n🔨 Building project ($InstallMode mode)..." -ForegroundColor Yellow
cd $ProjectPath
if ($InstallMode -eq "debug") {
    cargo build
} else {
    cargo build --release
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "✗ Build failed" -ForegroundColor Red
    exit 1
}

# Step 6: Verify Installation
Write-Host "`n✅ Verifying installation..." -ForegroundColor Yellow
$binary = if ($InstallMode -eq "debug") { 
    "target\debug\agentic_rag.exe" 
} else { 
    "target\release\agentic_rag.exe" 
}

if (Test-Path $binary) {
    Write-Host "✓ Binary created: $binary"
} else {
    Write-Host "✗ Binary not found" -ForegroundColor Red
    exit 1
}

Write-Host "`n" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "✅ Installation Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Start application: cargo run --release"
Write-Host "2. Visit: http://127.0.0.1:3000"
Write-Host "3. Check logs: cat $configDir\logs\backend.log.*"
Write-Host "4. Run tests: cargo test"
Write-Host "`n"
```

#### Running Windows Installation

```powershell
# Download/save script as "agentic-rag-install.ps1"

# Allow script execution (one-time)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Run installer
.\agentic-rag-install.ps1 -InstallMode release
```

### Linux Installation

#### Prerequisites (Ubuntu/Debian)

```bash
# Update package manager
sudo apt-get update

# Install build essentials
sudo apt-get install -y build-essential pkg-config libssl-dev git

# Verify Rust
rustc --version
cargo --version
```

#### Installation Script (Bash)

```bash
#!/bin/bash
# agentic-rag-install.sh
# Make executable: chmod +x agentic-rag-install.sh

PROJECT_PATH="${1:-.}"
INSTALL_MODE="${2:-release}"

echo "🚀 Agentic RAG Installer for Linux"
echo "Version: 1.0.0"
echo ""

# Step 1: Verify Prerequisites
echo "📋 Verifying prerequisites..."
if ! command -v rustc &> /dev/null; then
    echo "✗ Rust not found. Install from https://rustup.rs"
    exit 1
fi
echo "✓ Rust: $(rustc --version)"
echo "✓ Cargo: $(cargo --version)"

# Step 2: Create Directories
echo ""
echo "📁 Creating configuration directories..."
mkdir -p ~/.agentic-rag/{logs,data,uploads,models}
chmod 755 ~/.agentic-rag
echo "✓ Created: ~/.agentic-rag"

# Step 3: Set Environment Variables
echo ""
echo "🔧 Setting environment variables..."
if grep -q "RUST_LOG" ~/.bashrc; then
    echo "⚠ Environment variables already in ~/.bashrc"
else
    cat >> ~/.bashrc << 'EOF'

# Agentic RAG Configuration
export RUST_LOG=info
export MONITORING_ENABLED=true
export LOG_RETENTION_DAYS=7
export LOG_FORMAT=json
export AGENTIC_RAG_HOME=$HOME/.agentic-rag
EOF
    echo "✓ Environment variables added to ~/.bashrc"
    source ~/.bashrc
fi

# Step 4: Update Rust
echo ""
echo "📦 Updating Rust toolchain..."
rustup update

# Step 5: Build Project
echo ""
echo "🔨 Building project ($INSTALL_MODE mode)..."
cd "$PROJECT_PATH" || exit 1
if [ "$INSTALL_MODE" = "debug" ]; then
    cargo build
else
    cargo build --release
fi

if [ $? -ne 0 ]; then
    echo "✗ Build failed"
    exit 1
fi

# Step 6: Verify Installation
echo ""
echo "✅ Verifying installation..."
if [ "$INSTALL_MODE" = "debug" ]; then
    BINARY="target/debug/agentic_rag"
else
    BINARY="target/release/agentic_rag"
fi

if [ -f "$BINARY" ]; then
    echo "✓ Binary created: $BINARY"
    ls -lh "$BINARY"
else
    echo "✗ Binary not found"
    exit 1
fi

echo ""
echo "========================================"
echo "✅ Installation Complete!"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. Start application: cargo run --release"
echo "2. Visit: http://127.0.0.1:3000"
echo "3. Check logs: tail ~/.agentic-rag/logs/backend.log.*"
echo "4. Run tests: cargo test"
echo ""
```

#### Running Linux Installation

```bash
chmod +x agentic-rag-install.sh
./agentic-rag-install.sh . release
```

### macOS Installation

#### Prerequisites

```bash
# Install Xcode Command Line Tools (if not installed)
xcode-select --install

# Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install rust pkg-config openssl

# Verify Rust
rustc --version
cargo --version
```

#### Installation Script (Bash)

```bash
#!/bin/bash
# agentic-rag-install-macos.sh

PROJECT_PATH="${1:-.}"
INSTALL_MODE="${2:-release}"

echo "🚀 Agentic RAG Installer for macOS"
echo "Version: 1.0.0"
echo ""

# Step 1: Verify Prerequisites
echo "📋 Verifying prerequisites..."
if ! command -v rustc &> /dev/null; then
    echo "✗ Rust not found"
    echo "Install from https://rustup.rs or: brew install rust"
    exit 1
fi
echo "✓ Rust: $(rustc --version)"

# Step 2: Create Directories
echo ""
echo "📁 Creating configuration directories..."
mkdir -p ~/.agentic-rag/{logs,data,uploads,models}
chmod 755 ~/.agentic-rag
echo "✓ Created: ~/.agentic-rag"

# Step 3: Set Environment Variables
echo ""
echo "🔧 Setting environment variables..."
SHELL_RC=~/.zshrc  # macOS uses zsh by default
if [ -f "$SHELL_RC" ]; then
    if ! grep -q "RUST_LOG" "$SHELL_RC"; then
        cat >> "$SHELL_RC" << 'EOF'

# Agentic RAG Configuration
export RUST_LOG=info
export MONITORING_ENABLED=true
export LOG_RETENTION_DAYS=7
export LOG_FORMAT=json
export AGENTIC_RAG_HOME=$HOME/.agentic-rag
EOF
        source "$SHELL_RC"
    fi
fi
echo "✓ Environment variables configured"

# Step 4: Update Rust
echo ""
echo "📦 Updating Rust toolchain..."
rustup update

# Step 5: Build Project
echo ""
echo "🔨 Building project ($INSTALL_MODE mode)..."
cd "$PROJECT_PATH" || exit 1
if [ "$INSTALL_MODE" = "debug" ]; then
    cargo build
else
    cargo build --release
fi

if [ $? -ne 0 ]; then
    echo "✗ Build failed"
    exit 1
fi

# Step 6: Verify Installation
echo ""
echo "✅ Verifying installation..."
if [ "$INSTALL_MODE" = "debug" ]; then
    BINARY="target/debug/agentic_rag"
else
    BINARY="target/release/agentic_rag"
fi

if [ -f "$BINARY" ]; then
    echo "✓ Binary created: $BINARY"
    ls -lh "$BINARY"
else
    echo "✗ Binary not found"
    exit 1
fi

echo ""
echo "========================================"
echo "✅ Installation Complete!"
echo "========================================"
echo ""
echo "Next steps:"
echo "1. Start application: cargo run --release"
echo "2. Visit: http://127.0.0.1:3000"
echo "3. Check logs: tail ~/.agentic-rag/logs/backend.log.*"
echo "4. Run tests: cargo test"
echo ""
```

---

## CONFIGURATION

### Default Configuration File

**Location**: `~/.agentic-rag/config.toml`

```toml
# Agentic RAG Configuration
# Version: 1.0.0
# Auto-generated during installation

[server]
# Server binding
host = "127.0.0.1"
port = 3000
worker_threads = 4
request_timeout_secs = 30
max_payload_size_mb = 50

[monitoring]
# Monitoring and observability
enabled = true
log_level = "info"           # debug, info, warn, error
log_format = "json"          # json or text
metrics_enabled = true
health_check_interval_secs = 30
trace_enabled = true

[logging]
# Log file configuration
file_path = "~/.agentic-rag/logs"
max_log_files = 7            # Keep 7 days of logs
rotation = "daily"           # daily rotation
log_retention_days = 7

[storage]
# Data storage paths
data_path = "~/.agentic-rag/data"
uploads_path = "~/.agentic-rag/uploads"
models_path = "~/.agentic-rag/models"
cache_path = "~/.agentic-rag/cache"

[embedding]
# Embedding model configuration
model = "all-MiniLM-L6-v2"
batch_size = 32
cache_enabled = true

[rag]
# RAG pipeline configuration
chunk_size = 512
chunk_overlap = 75
top_k = 5
similarity_threshold = 0.7

[features]
# Feature flags
rag_enabled = true
monitoring_enabled = true
health_checks_enabled = true
metrics_export_enabled = true
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Logging level (debug, info, warn, error) |
| `MONITORING_ENABLED` | `true` | Enable monitoring layer |
| `LOG_RETENTION_DAYS` | `7` | Keep logs for N days |
| `LOG_FORMAT` | `json` | Log format (json or text) |
| `AGENTIC_RAG_HOME` | `~/.agentic-rag` | Installation home directory |
| `SERVER_PORT` | `3000` | Server port |
| `SERVER_HOST` | `127.0.0.1` | Server host binding |

### Quick Configuration Presets

#### Development
```bash
export RUST_LOG=debug
export MONITORING_ENABLED=true
export LOG_FORMAT=text
```

#### Production
```bash
export RUST_LOG=info,tantivy=warn
export MONITORING_ENABLED=true
export LOG_FORMAT=json
export LOG_RETENTION_DAYS=7
```

#### Testing
```bash
export RUST_LOG=debug
export MONITORING_ENABLED=false
export LOG_FORMAT=text
```

---

## POST-INSTALLATION VERIFICATION

### Health Check (1 minute)

**Start the application:**

```bash
cd project-root
cargo run --release
```

Expected startup output:
```
2025-11-03T12:00:00Z INFO  agentic_rag::config: Initializing configuration
2025-11-03T12:00:00Z INFO  agentic_rag::database: Connecting to database
2025-11-03T12:00:01Z INFO  agentic_rag::api: Server listening on http://127.0.0.1:3000
2025-11-03T12:00:01Z INFO  agentic_rag::monitoring: Monitoring initialized
```

**In another terminal, verify endpoints:**

```bash
# Health check
curl http://127.0.0.1:3000/monitoring/health

# Readiness check
curl http://127.0.0.1:3000/monitoring/ready

# Liveness check
curl http://127.0.0.1:3000/monitoring/live

# Metrics
curl http://127.0.0.1:3000/monitoring/metrics
```

Expected responses:
```json
{
  "status": "healthy",
  "timestamp": "2025-11-03T12:00:01Z",
  "components": {
    "api": "healthy",
    "database": "healthy",
    "monitoring": "healthy"
  }
}
```

### Full Verification Checklist

```
Directory Structure
  ☐ ~/.agentic-rag directory exists
  ☐ logs directory writable
  ☐ data directory writable
  ☐ uploads directory writable
  ☐ models directory writable

Configuration
  ☐ config.toml created
  ☐ Environment variables set
  ☐ Ports 3000 not in use

Application Start
  ☐ Application starts without errors
  ☐ No compilation warnings
  ☐ Startup time < 5 seconds

API Endpoints
  ☐ /monitoring/health returns 200
  ☐ /monitoring/ready returns 200
  ☐ /monitoring/live returns 200
  ☐ /monitoring/metrics returns 200

Logging
  ☐ Log files created in ~/.agentic-rag/logs/
  ☐ Logs contain startup messages
  ☐ Log rotation configured

Web Interface
  ☐ Frontend loads at http://127.0.0.1:3000
  ☐ Browser console shows no errors
  ☐ CSS/JS assets loaded

Test Suite
  ☐ Unit tests pass: cargo test --lib
  ☐ Integration tests pass: cargo test --test '*'
  ☐ No test failures
```

---

## MONITORING & HEALTH CHECKS

### Health Endpoint Details

**GET `/monitoring/health`** - Full health status

Response:
```json
{
  "status": "healthy",
  "timestamp": "2025-11-03T12:30:45Z",
  "uptime_seconds": 1830,
  "components": {
    "api": {
      "status": "healthy",
      "uptime_seconds": 1830
    },
    "database": {
      "status": "healthy",
      "connections": 5
    },
    "monitoring": {
      "status": "healthy",
      "metrics_collected": 1250
    }
  }
}
```

**GET `/monitoring/ready`** - Readiness probe (Kubernetes compatible)

Returns 200 if all components are ready, 503 otherwise.

**GET `/monitoring/live`** - Liveness probe (Kubernetes compatible)

Returns 200 if application is running, 503 otherwise.

**GET `/monitoring/metrics`** - Prometheus metrics

Returns metrics in Prometheus text format:
```
# HELP startup_duration_ms Application startup time
# TYPE startup_duration_ms gauge
startup_duration_ms 2450

# HELP api_request_duration_ms API request latency
# TYPE api_request_duration_ms histogram
api_request_duration_ms_bucket{endpoint="/search",le="100"} 45
api_request_duration_ms_bucket{endpoint="/search",le="500"} 98
```

### Monitoring Dashboard

View logs in real-time:

**Windows (PowerShell):**
```powershell
Get-Content -Path "$env:USERPROFILE\.agentic-rag\logs\backend.log.*" -Tail 20 -Wait
```

**Linux/macOS:**
```bash
tail -f ~/.agentic-rag/logs/backend.log.*
```

### Metrics Tracked

| Metric | Type | Description |
|--------|------|-------------|
| `startup_duration_ms` | Gauge | Initial startup time |
| `api_request_duration_ms` | Histogram | API endpoint latency |
| `database_query_duration_ms` | Histogram | Database query time |
| `api_errors_total` | Counter | Total API errors |
| `config_load_duration_ms` | Gauge | Configuration load time |
| `memory_usage_bytes` | Gauge | Process memory usage |
| `http_requests_total` | Counter | Total HTTP requests |

---

## TROUBLESHOOTING

### Common Installation Issues

#### ❌ "Rust not found"

**Solution:**
```bash
# Install Rust from https://rustup.rs
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### ❌ "cargo: command not found"

**Windows:** Restart terminal/IDE after Rust installation

**Linux/macOS:**
```bash
source $HOME/.cargo/env
```

#### ❌ "Build fails with linking errors"

**Windows:**
- Ensure Visual Studio Build Tools are installed
- Run installer as Administrator

**Linux:**
```bash
sudo apt-get install build-essential pkg-config libssl-dev
```

**macOS:**
```bash
xcode-select --install
brew install openssl
```

#### ❌ "Port 3000 already in use"

**Windows (PowerShell):**
```powershell
netstat -ano | findstr :3000
taskkill /PID <PID> /F  # Kill process using port
```

**Linux/macOS:**
```bash
lsof -i :3000
kill -9 <PID>  # Kill process using port
```

Or change port in config:
```toml
[server]
port = 3001  # Use different port
```

#### ❌ "Cannot create ~/.agentic-rag directory"

**Solution:**
- Verify home directory is writable
- Check file permissions
- Ensure not running in restricted mode

```bash
touch ~/test.txt  # Test write access
rm ~/test.txt
```

#### ❌ "Health endpoint returns 503"

**Solution:**
1. Check logs: `tail ~/.agentic-rag/logs/backend.log.*`
2. Verify database connection
3. Verify all ports are available
4. Check for startup errors

### Runtime Issues

#### ❌ "Application crashes on startup"

**Debug steps:**
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Check logs
cat ~/.agentic-rag/logs/backend.log.*

# Verify configuration
cat ~/.agentic-rag/config.toml
```

#### ❌ "Very slow startup (>10 seconds)"

**Solution:**
- First startup compiles dependencies (normal)
- Subsequent startups are fast
- If persistent, check disk I/O and system resources

#### ❌ "High memory usage"

**Monitor:**
```bash
# Linux/macOS
ps aux | grep agentic_rag

# Windows
tasklist | findstr agentic_rag
```

**Solution:**
- Reduce log retention: `LOG_RETENTION_DAYS=3`
- Reduce metric collection interval
- Check for memory leaks in logs

#### ❌ "Logs not created"

**Verify:**
```bash
ls -la ~/.agentic-rag/logs/
```

**Solution:**
- Check directory permissions
- Verify `LOG_RETENTION_DAYS` is not 0
- Restart application

### Verification & Debugging

#### Test API Connectivity

```bash
# Test connectivity
curl -v http://127.0.0.1:3000/monitoring/health

# Test with response headers
curl -i http://127.0.0.1:3000/monitoring/health

# Test with timing
curl -w "\nTotal time: %{time_total}s\n" http://127.0.0.1:3000/monitoring/health
```

#### View Detailed Logs

```bash
# Most recent log entry (last 50 lines)
tail -50 ~/.agentic-rag/logs/backend.log.*

# Search logs for errors
grep "ERROR" ~/.agentic-rag/logs/backend.log.*

# Follow logs in real-time
tail -f ~/.agentic-rag/logs/backend.log.*

# Count log entries
wc -l ~/.agentic-rag/logs/backend.log.*
```

#### Check System Resources

**Linux/macOS:**
```bash
# CPU usage
top -p $(pgrep -f agentic_rag)

# Memory usage
ps aux | grep agentic_rag

# Disk usage
du -sh ~/.agentic-rag/
```

**Windows:**
```powershell
Get-Process | Where-Object {$_.ProcessName -like "*agentic*"} | Format-List
```

---

## UNINSTALLATION

### Full Uninstallation

**Windows (PowerShell):**
```powershell
# Remove application binary
Remove-Item -Path "target" -Recurse -Force

# Remove configuration and data (CAUTION - deletes data)
Remove-Item -Path "$env:USERPROFILE\.agentic-rag" -Recurse -Force

# Remove environment variables
[Environment]::SetEnvironmentVariable("RUST_LOG", $null, "User")
[Environment]::SetEnvironmentVariable("MONITORING_ENABLED", $null, "User")
[Environment]::SetEnvironmentVariable("LOG_RETENTION_DAYS", $null, "User")
[Environment]::SetEnvironmentVariable("AGENTIC_RAG_HOME", $null, "User")

Write-Host "✓ Uninstallation complete"
```

**Linux/macOS:**
```bash
# Remove application binary and build artifacts
rm -rf target

# Remove configuration and data (CAUTION - deletes data)
rm -rf ~/.agentic-rag

# Remove from shell configuration
sed -i '/Agentic RAG Configuration/,/AGENTIC_RAG_HOME/d' ~/.bashrc

echo "✓ Uninstallation complete"
```

### Backup Before Uninstalling

```bash
# Create backup
tar -czf ~/agentic-rag-backup-$(date +%Y%m%d).tar.gz ~/.agentic-rag/

# Verify backup
tar -tzf ~/agentic-rag-backup-*.tar.gz | head -20
```

### Partial Uninstallation (Keep Data)

**Windows:**
```powershell
# Remove only binaries
Remove-Item -Path "target" -Recurse -Force

# Keep ~/.agentic-rag for data preservation
```

**Linux/macOS:**
```bash
# Remove only binaries
rm -rf target

# Keep ~/.agentic-rag for data preservation
```

---

## INSTALLER INTEGRATION POINTS

### Key Integration Requirements

The installer must handle these critical integration points:

#### 1. Directory Creation

**Must create and verify:**
- `~/.agentic-rag/` - Main config directory
- `~/.agentic-rag/logs/` - Log directory (writable)
- `~/.agentic-rag/data/` - Data directory (writable)
- `~/.agentic-rag/uploads/` - Upload directory (writable)
- `~/.agentic-rag/models/` - Models directory (writable)

**Verification code:**
```bash
#!/bin/bash
for dir in logs data uploads models; do
  if [ ! -d "$HOME/.agentic-rag/$dir" ]; then
    echo "✗ Missing: $dir"
    exit 1
  fi
done
echo "✓ All directories verified"
```

#### 2. Environment Variables

**Must set and persist:**
- `RUST_LOG=info`
- `MONITORING_ENABLED=true`
- `LOG_RETENTION_DAYS=7`
- `LOG_FORMAT=json`
- `AGENTIC_RAG_HOME=~/.agentic-rag`

**Verification code:**
```bash
#!/bin/bash
env | grep -E "RUST_LOG|MONITORING_ENABLED|AGENTIC_RAG_HOME" || {
  echo "✗ Environment variables not set"
  exit 1
}
```

#### 3. Health Verification

**Installer must verify startup:**

```bash
#!/bin/bash
# Start application in background
cargo run --release &
APP_PID=$!

# Wait for startup
sleep 3

# Check health endpoint
HEALTH=$(curl -s http://127.0.0.1:3000/monitoring/health)
STATUS=$(echo $HEALTH | jq -r '.status')

if [ "$STATUS" != "healthy" ]; then
  echo "✗ Health check failed"
  kill $APP_PID
  exit 1
fi

echo "✓ Application healthy"
kill $APP_PID
```

#### 4. Log Verification

**Installer must verify logging:**

```bash
#!/bin/bash
# Check if logs were created
if [ ! -f "$HOME/.agentic-rag/logs/backend.log."* ]; then
  echo "✗ No log files created"
  exit 1
fi

# Verify log content
if grep -q "listening" "$HOME/.agentic-rag/logs/backend.log."*; then
  echo "✓ Logs verified"
else
  echo "✗ Logs not properly written"
  exit 1
fi
```

#### 5. Performance Measurement

**Installer should record:**
- Installation start time
- Build completion time
- First startup time
- Health check response time

```bash
#!/bin/bash
START=$(date +%s)

# ... installation steps ...

END=$(date +%s)
DURATION=$((END - START))

echo "Installation completed in ${DURATION}s"
```

#### 6. Installer Logging

**Create installer-specific log:**

```bash
#!/bin/bash
INSTALLER_LOG="$HOME/.agentic-rag/installer.log"

{
  echo "=== Agentic RAG Installation Log ==="
  echo "Date: $(date)"
  echo "Platform: $(uname -s)"
  echo "Rust: $(rustc --version)"
  echo "Cargo: $(cargo --version)"
  echo ""
  # ... installer steps logged ...
  echo "Installation status: SUCCESS"
} | tee "$INSTALLER_LOG"
```

### Installer Checklist

```
Pre-Installation
  ☐ Verify Rust/Cargo installed
  ☐ Verify system requirements met
  ☐ Verify network connectivity
  ☐ Create installer log file

Directory Setup
  ☐ Create ~/.agentic-rag/ directory
  ☐ Create logs/ subdirectory
  ☐ Create data/ subdirectory
  ☐ Create uploads/ subdirectory
  ☐ Create models/ subdirectory
  ☐ Set correct permissions (755)

Environment Configuration
  ☐ Set RUST_LOG environment variable
  ☐ Set MONITORING_ENABLED=true
  ☐ Set LOG_RETENTION_DAYS=7
  ☐ Set LOG_FORMAT=json
  ☐ Persist environment variables

Build & Compilation
  ☐ Verify Cargo.toml exists
  ☐ Run cargo check
  ☐ Run cargo build --release
  ☐ Verify binary created
  ☐ Record build time

Initial Configuration
  ☐ Create config.toml if missing
  ☐ Verify configuration syntax
  ☐ Check port availability (3000)

Post-Build Verification
  ☐ Start application (timeout 10s)
  ☐ Wait for startup (max 5s)
  ☐ Check health endpoint
  ☐ Verify status is "healthy"
  ☐ Verify logs created
  ☐ Stop application

Testing
  ☐ Run unit tests
  ☐ Run integration tests
  ☐ Verify test output
  ☐ Log any failures

Cleanup
  ☐ Remove temporary files
  ☐ Close open handles
  ☐ Kill background processes

Success Reporting
  ☐ Log installation complete message
  ☐ Display next steps
  ☐ Save installer log
  ☐ Return exit code 0 on success
```

---

## GETTING HELP

### Documentation
- 📖 **Installation Issues**: See [Troubleshooting](#troubleshooting)
- 📖 **Configuration**: See [Configuration](#configuration)
- 📖 **Health Checks**: See [Monitoring & Health Checks](#monitoring--health-checks)

### Online Resources
- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Documentation**: https://doc.rust-lang.org/cargo/
- **Actix Web**: https://actix.rs/
- **Dioxus**: https://dioxuslabs.com/

### Log Locations

| File | Location |
|------|----------|
| **Backend Log** | `~/.agentic-rag/logs/backend.log.YYYY-MM-DD` |
| **Installer Log** | `~/.agentic-rag/installer.log` |
| **Application Log** | stdout (when running with `cargo run`) |

---

## VERSION HISTORY

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-03 | Initial release; Windows, Linux, macOS support |

---

## QUICK START REFERENCE

### 30-Second Quick Start

```bash
# 1. Ensure Rust is installed
rustc --version

# 2. Clone project
git clone <repo> && cd agentic-rag

# 3. Install (all platforms)
# Choose: Windows (PowerShell), Linux (Bash), or macOS (Bash)
# Follow platform-specific section above

# 4. Run
cargo run --release

# 5. Verify
curl http://127.0.0.1:3000/monitoring/health
```

### Common Commands

```bash
# Start application
cargo run --release

# Run tests
cargo test

# View logs
tail -f ~/.agentic-rag/logs/backend.log.*

# Check health
curl http://127.0.0.1:3000/monitoring/health

# Stop application
Ctrl+C
```

---

**Installation Guide v1.0.0**  
**Agentic RAG System**  
**Created**: 2025-11-03  
**Status**: Ready for Deployment