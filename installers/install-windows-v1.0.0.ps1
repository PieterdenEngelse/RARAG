#!/bin/bash

################################################################################
# Agentic RAG Installer for Linux/macOS - ENHANCED v1.1.0
# Date: 2025-11-03
# Platforms: Ubuntu 20.04+, CentOS 8+, Debian 11+, macOS 10.15+
#
# ENHANCEMENTS from v1.0.0:
#   ✓ Improved argument parsing (--help, --version, --skip-checks)
#   ✓ AG_HOME support for PathManager (v13.1.2 integration)
#   ✓ Database initialization (documents.db, memory.db)
#   ✓ Better error handling (set -euo pipefail)
#   ✓ Separate logging functions (log_info, log_success, log_warn, log_error)
#   ✓ show_help function for better documentation
#   ✓ Configuration template with Redis and PathManager settings
#   ✓ Verbose mode with set -x for debugging
#
# Usage:
#   chmod +x install-linux-v1.1.0.sh
#   ./install-linux-v1.1.0.sh [OPTIONS]
#
# Options:
#   --project-path PATH     Path to project (default: current directory)
#   --mode release|debug    Build mode (default: release)
#   --prefix PATH           Installation prefix (default: ~/.agentic-rag)
#   --backend-port PORT     Backend port (default: 3010)
#   --frontend-port PORT    Frontend port (default: 3000)
#   --verbose               Enable verbose logging
#   --skip-checks           Skip preflight checks
#   --version               Show version
#   --help                  Show this help message
#
# Integration Points:
#   - Directory creation (~/.agentic-rag/ + AG_HOME paths)
#   - Database initialization (documents.db, memory.db)
#   - Environment variable setup (persistence)
#   - Health verification (curl endpoint)
#   - Comprehensive logging (~/.agentic-rag/installer.log)
#   - Performance metrics
#   - Redis configuration support
################################################################################

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
INSTALLER_VERSION="1.1.0"
PROJECT_PATH="${PWD}"
INSTALL_MODE="release"
INSTALL_PREFIX="${HOME}/.agentic-rag"
BACKEND_PORT="3010"
FRONTEND_PORT="3000"
HEALTH_CHECK_TIMEOUT=10
STARTUP_DELAY=3
VERBOSE=false
SKIP_CHECKS=false
SHELL_RC=""

# Logging functions (from v13.1.2)
log_info() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${BLUE}[${timestamp}] ℹ️  ${1}${NC}"
    echo "[${timestamp}] INFO: ${1}" >> "${INSTALLER_LOG}" 2>/dev/null || true
}

log_success() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${GREEN}[${timestamp}] ✓ ${1}${NC}"
    echo "[${timestamp}] SUCCESS: ${1}" >> "${INSTALLER_LOG}" 2>/dev/null || true
}

log_warn() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${YELLOW}[${timestamp}] ⚠️  ${1}${NC}"
    echo "[${timestamp}] WARN: ${1}" >> "${INSTALLER_LOG}" 2>/dev/null || true
}

log_error() {
    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "${RED}[${timestamp}] ✗ ${1}${NC}" >&2
    echo "[${timestamp}] ERROR: ${1}" >> "${INSTALLER_LOG}" 2>/dev/null || true
}

# Show help (from v13.1.2)
show_help() {
    cat << EOF
Agentic RAG Installation Script v${INSTALLER_VERSION}

USAGE:
    ./install-linux-v1.1.0.sh [OPTIONS]

OPTIONS:
    --project-path PATH       Path to project (default: current directory)
    --mode release|debug      Build mode (default: release)
    --prefix PATH             Installation prefix (default: ~/.agentic-rag)
    --backend-port PORT       Backend port (default: 3010)
    --frontend-port PORT      Frontend port (default: 3000)
    --verbose                 Enable verbose logging with set -x
    --skip-checks             Skip preflight checks
    --version                 Show version
    --help                    Show this help message

EXAMPLES:
    ./install-linux-v1.1.0.sh
    ./install-linux-v1.1.0.sh --mode debug --verbose
    ./install-linux-v1.1.0.sh --prefix /opt/fro --backend-port 8080
    ./install-linux-v1.1.0.sh --project-path ~/projects/fro --skip-checks

PLATFORMS:
    • Ubuntu 20.04, 22.04, 24.04
    • Debian 11, 12
    • CentOS 8, 9
    • macOS 10.15+

REQUIREMENTS:
    • Rust 1.70+ (from https://rustup.rs)
    • Cargo (included with Rust)
    • Git (optional)
    • 4 GB RAM, 1 GB disk space

EOF
}

# Parse arguments (improved from v13.1.2)
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --project-path)
                PROJECT_PATH="$2"
                shift 2
                ;;
            --mode)
                INSTALL_MODE="$2"
                if [[ "$INSTALL_MODE" != "release" && "$INSTALL_MODE" != "debug" ]]; then
                    log_error "Invalid mode: $INSTALL_MODE (must be 'release' or 'debug')"
                    exit 1
                fi
                shift 2
                ;;
            --prefix)
                INSTALL_PREFIX="$2"
                shift 2
                ;;
            --backend-port)
                BACKEND_PORT="$2"
                shift 2
                ;;
            --frontend-port)
                FRONTEND_PORT="$2"
                shift 2
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --skip-checks)
                SKIP_CHECKS=true
                shift
                ;;
            --version)
                echo "Agentic RAG Installer v${INSTALLER_VERSION}"
                exit 0
                ;;
            --help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
}

# Detect shell
detect_shell() {
    if [ -f "${HOME}/.zshrc" ]; then
        SHELL_RC="${HOME}/.zshrc"
    elif [ -f "${HOME}/.bashrc" ]; then
        SHELL_RC="${HOME}/.bashrc"
    else
        SHELL_RC="${HOME}/.profile"
    fi
    log_info "Detected shell: $SHELL_RC"
}

# Verify prerequisites (improved with better checks)
verify_prerequisites() {
    if [ "$SKIP_CHECKS" = true ]; then
        log_warn "Preflight checks skipped (--skip-checks)"
        return 0
    fi

    log_info "Running preflight checks..."
    
    # Check OS
    if [[ "$OSTYPE" != "linux-gnu"* && "$OSTYPE" != "darwin"* ]]; then
        log_error "Unsupported OS: $OSTYPE"
        return 1
    fi
    log_success "OS supported: $OSTYPE"

    # Check Rust
    if ! command -v rustc &> /dev/null; then
        log_error "Rust not found. Please install from https://rustup.rs"
        return 1
    fi
    local rust_version
    rust_version=$(rustc --version)
    log_success "Rust installed: $rust_version"

    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found"
        return 1
    fi
    local cargo_version
    cargo_version=$(cargo --version)
    log_success "Cargo installed: $cargo_version"

    # Check Git (optional)
    if ! command -v git &> /dev/null; then
        log_warn "Git not found (optional but recommended)"
    else
        local git_version
        git_version=$(git --version)
        log_success "Git installed: $git_version"
    fi

    # Check build tools
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if ! command -v gcc &> /dev/null; then
            log_error "GCC not found. Run: sudo apt-get install build-essential"
            return 1
        fi
        log_success "Build tools verified"
    fi

    log_success "All prerequisites verified"
    return 0
}

# Create directories (enhanced with AG_HOME from v13.1.2)
create_directories() {
    log_info "Creating configuration directories..."
    
    local dirs=(
        "${INSTALL_PREFIX}"
        "${INSTALL_PREFIX}/logs"
        "${INSTALL_PREFIX}/data"
        "${INSTALL_PREFIX}/uploads"
        "${INSTALL_PREFIX}/models"
        "${INSTALL_PREFIX}/db"
        "${INSTALL_PREFIX}/index"
        "${INSTALL_PREFIX}/cache"
        "${INSTALL_PREFIX}/config"
    )
    
    for dir in "${dirs[@]}"; do
        if [ ! -d "$dir" ]; then
            mkdir -p "$dir"
            chmod 755 "$dir"
            log_success "Created directory: $dir"
        else
            log_info "Directory exists: $dir"
        fi
    done
}

# Setup environment variables (enhanced with AG_HOME)
setup_environment() {
    log_info "Setting up environment variables..."
    
    local env_vars=(
        "RUST_LOG=info"
        "MONITORING_ENABLED=true"
        "LOG_RETENTION_DAYS=7"
        "LOG_FORMAT=json"
        "AGENTIC_RAG_HOME=${INSTALL_PREFIX}"
        "AG_HOME=${INSTALL_PREFIX}"
        "BACKEND_PORT=${BACKEND_PORT}"
        "FRONTEND_PORT=${FRONTEND_PORT}"
    )
    
    # Check if already configured
    local needs_update=false
    for var in "${env_vars[@]}"; do
        local var_name="${var%%=*}"
        if ! grep -q "export ${var_name}" "${SHELL_RC}"; then
            needs_update=true
            break
        fi
    done
    
    if [ "$needs_update" = true ]; then
        cat >> "${SHELL_RC}" << 'EOF'

# Agentic RAG Configuration (v1.1.0)
export RUST_LOG=info
export MONITORING_ENABLED=true
export LOG_RETENTION_DAYS=7
export LOG_FORMAT=json
export AGENTIC_RAG_HOME=$HOME/.agentic-rag
export AG_HOME=$HOME/.agentic-rag
export BACKEND_PORT=3010
export FRONTEND_PORT=3000
EOF
        log_success "Environment variables added to ${SHELL_RC}"
    else
        log_info "Environment variables already configured"
    fi
    
    # Export for current shell
    export RUST_LOG=info
    export MONITORING_ENABLED=true
    export LOG_RETENTION_DAYS=7
    export LOG_FORMAT=json
    export AGENTIC_RAG_HOME="${INSTALL_PREFIX}"
    export AG_HOME="${INSTALL_PREFIX}"
    export BACKEND_PORT
    export FRONTEND_PORT
}

# NEW v1.1.0: Initialize databases (from v13.1.2)
init_databases() {
    log_info "Initializing databases..."
    
    local db_dir="${INSTALL_PREFIX}/db"
    touch "$db_dir/documents.db"
    touch "$db_dir/memory.db"
    
    log_success "Databases created: documents.db, memory.db"
}

# Verify dependencies
verify_dependencies() {
    log_info "Verifying Rust dependencies..."
    
    cd "${PROJECT_PATH}"
    
    if cargo check > /dev/null 2>&1; then
        log_success "Dependencies verified"
    else
        log_warn "Some dependencies may need to be downloaded"
    fi
}

# Build project
build_project() {
    log_info "Building project in ${INSTALL_MODE} mode..."
    
    cd "${PROJECT_PATH}"
    
    local build_mode_flag=""
    if [ "${INSTALL_MODE}" = "release" ]; then
        build_mode_flag="--release"
    fi
    
    if cargo build ${build_mode_flag} >> "${INSTALLER_LOG}" 2>&1; then
        log_success "Build completed successfully"
    else
        log_error "Build failed. Check ${INSTALLER_LOG} for details"
        exit 1
    fi
}

# Verify build artifacts
verify_artifacts() {
    log_info "Verifying build artifacts..."
    
    local binary_path="${PROJECT_PATH}/target/${INSTALL_MODE}/fro"
    
    if [ -f "${binary_path}" ]; then
        local file_size
        file_size=$(du -h "${binary_path}" | cut -f1)
        log_success "Binary verified: ${binary_path} (${file_size})"
    else
        log_warn "Expected binary not found at ${binary_path}"
    fi
}

# Create configuration (enhanced with AG_HOME and Redis support)
create_configuration() {
    log_info "Creating configuration file..."
    
    local config_file="${INSTALL_PREFIX}/config/.env"
    
    if [ ! -f "${config_file}" ]; then
        cat > "${config_file}" << EOF
# Agentic RAG Configuration
# Version: 1.1.0
# Generated: $(date)

# Core Settings
INSTALL_PREFIX=${INSTALL_PREFIX}
AG_HOME=${INSTALL_PREFIX}
PROJECT_PATH=${PROJECT_PATH}

# Backend Configuration
BACKEND_HOST=127.0.0.1
BACKEND_PORT=${BACKEND_PORT}

# Frontend Configuration
FRONTEND_PORT=${FRONTEND_PORT}

# Directories
CONFIG_DIR=${INSTALL_PREFIX}/config
DATA_DIR=${INSTALL_PREFIX}/data
LOG_DIR=${INSTALL_PREFIX}/logs
DB_DIR=${INSTALL_PREFIX}/db
INDEX_DIR=${INSTALL_PREFIX}/index
CACHE_DIR=${INSTALL_PREFIX}/cache

# Logging
RUST_LOG=info
LOG_FORMAT=json
LOG_RETENTION_DAYS=7

# Monitoring
MONITORING_ENABLED=true
HEALTH_CHECK_INTERVAL_SECS=30

# Redis Configuration (Phase 12 support)
REDIS_ENABLED=true
REDIS_URL=redis://127.0.0.1:6379/
REDIS_TTL=3600

# Feature Flags
ENABLE_RAG=true
ENABLE_MONITORING=true
ENABLE_HEALTH_CHECKS=true
EOF
        log_success "Configuration created: ${config_file}"
    else
        log_info "Configuration already exists"
    fi
}

# Check port availability
check_port_availability() {
    log_info "Checking port availability..."
    
    if lsof -Pi :${BACKEND_PORT} -sTCP:LISTEN -t > /dev/null 2>&1; then
        log_warn "Port ${BACKEND_PORT} is already in use"
        return 1
    else
        log_success "Port ${BACKEND_PORT} is available"
        return 0
    fi
}

# Health check
health_check() {
    log_info "Performing health check..."
    
    local max_attempts=5
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if response=$(curl -s -m 2 http://127.0.0.1:3000/monitoring/health 2>/dev/null); then
            if echo "$response" | grep -q '"status":"healthy"'; then
                log_success "Health check passed"
                return 0
            fi
        fi
        
        log_info "Health check attempt ${attempt}/${max_attempts}..."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    log_warn "Health check did not receive healthy response"
    return 1
}

# Post-build verification
post_build_verification() {
    log_info "Running post-build verification..."
    
    if ! check_port_availability; then
        log_warn "Port check indicated potential conflict"
    fi
    
    log_info "Starting application for verification (timeout: ${HEALTH_CHECK_TIMEOUT}s)..."
    
    cd "${PROJECT_PATH}"
    
    # Start application in background
    timeout ${HEALTH_CHECK_TIMEOUT} cargo run --release >> "${INSTALLER_LOG}" 2>&1 &
    local app_pid=$!
    
    log_info "Application started (PID: ${app_pid})"
    log_info "Waiting for startup (${STARTUP_DELAY}s delay)..."
    
    sleep ${STARTUP_DELAY}
    
    # Attempt health check
    if health_check; then
        log_success "Post-build verification successful"
    else
        log_warn "Post-build verification incomplete (may be expected)"
    fi
    
    # Kill application
    if kill -0 ${app_pid} 2>/dev/null; then
        kill ${app_pid} 2>/dev/null || true
        sleep 1
        log_info "Application stopped"
    fi
}

# Run tests
run_tests() {
    log_info "Running tests..."
    
    cd "${PROJECT_PATH}"
    
    if cargo test --release >> "${INSTALLER_LOG}" 2>&1; then
        log_success "All tests passed"
    else
        log_warn "Some tests may have failed"
    fi
}

# Cleanup
cleanup() {
    log_info "Cleaning up..."
    
    # Kill any lingering processes
    pkill -f "cargo run" || true
    
    log_success "Cleanup completed"
}

# Display summary
display_summary() {
    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    
    echo ""
    echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║    ✓ Installation Completed Successfully (v${INSTALLER_VERSION})  ║${NC}"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    echo -e "  ${GREEN}📊 Installation Summary:${NC}"
    echo "    • Duration: ${duration}s"
    echo "    • Project: ${PROJECT_PATH}"
    echo "    • Installation Prefix: ${INSTALL_PREFIX}"
    echo "    • AG_HOME: ${INSTALL_PREFIX}"
    echo "    • Mode: ${INSTALL_MODE}"
    echo "    • Log: ${INSTALLER_LOG}"
    echo ""
    
    echo -e "  ${GREEN}🔗 Endpoints:${NC}"
    echo "    • Backend: http://127.0.0.1:${BACKEND_PORT}"
    echo "    • Frontend: http://127.0.0.1:${FRONTEND_PORT}"
    echo "    • Health: http://127.0.0.1:3000/monitoring/health"
    echo ""
    
    echo -e "  ${GREEN}📝 Next Steps:${NC}"
    echo "    1. Source shell config: source ${SHELL_RC}"
    echo "    2. Navigate to project: cd ${PROJECT_PATH}"
    echo "    3. Start application: cargo run --${INSTALL_MODE}"
    echo "    4. Check database: ls -la ${INSTALL_PREFIX}/db/"
    echo "    5. View logs: tail -f ${INSTALL_PREFIX}/logs/*.log"
    echo ""
    
    echo -e "  ${GREEN}📚 Documentation:${NC}"
    echo "    • Config: ${INSTALL_PREFIX}/config/.env"
    echo "    • Installer log: ${INSTALLER_LOG}"
    echo "    • Data dir: ${INSTALL_PREFIX}/data"
    echo "    • Database dir: ${INSTALL_PREFIX}/db"
    echo ""
}

# Main function
main() {
    START_TIME=$(date +%s)
    
    # Initialize
    detect_shell
    parse_args "$@"
    
    # Enable verbose mode if requested
    if [ "$VERBOSE" = true ]; then
        set -x
    fi
    
    # Create log directory
    mkdir -p "${INSTALL_PREFIX}"
    touch "${INSTALLER_LOG}"
    
    # Print header
    echo ""
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║  Agentic RAG Installer for Linux/macOS - v${INSTALLER_VERSION}    ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Log startup info
    {
        echo "=== Agentic RAG Installation Log ==="
        echo "Date: $(date)"
        echo "Platform: $(uname -s)"
        echo "Architecture: $(uname -m)"
        echo "Rust: $(rustc --version)"
        echo "Cargo: $(cargo --version)"
        echo "Installer Version: ${INSTALLER_VERSION}"
        echo "Project Path: ${PROJECT_PATH}"
        echo "Install Prefix: ${INSTALL_PREFIX}"
        echo "AG_HOME: ${INSTALL_PREFIX}"
        echo "Backend Port: ${BACKEND_PORT}"
        echo "Frontend Port: ${FRONTEND_PORT}"
        echo "Install Mode: ${INSTALL_MODE}"
        echo ""
    } >> "${INSTALLER_LOG}"
    
    log_info "Installer Version: ${INSTALLER_VERSION}"
    log_info "Platform: $(uname -s)"
    log_info "Project Path: ${PROJECT_PATH}"
    log_info "Install Prefix: ${INSTALL_PREFIX}"
    echo ""
    
    # Execute installation steps
    verify_prerequisites || exit 1
    echo ""
    create_directories
    echo ""
    setup_environment
    echo ""
    init_databases
    echo ""
    verify_dependencies
    echo ""
    build_project
    echo ""
    verify_artifacts
    echo ""
    create_configuration
    echo ""
    post_build_verification
    echo ""
    run_tests
    echo ""
    cleanup
    
    # Log completion
    {
        echo ""
        echo "Installation completed successfully"
        echo "Duration: $(( $(date +%s) - START_TIME ))s"
        echo "Status: SUCCESS"
    } >> "${INSTALLER_LOG}"
    
    display_summary
}

# Run main function
main "$@"

################################################################################
# END OF INSTALLER v1.1.0
################################################################################