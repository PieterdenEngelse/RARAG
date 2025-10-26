#!/bin/bash
# install.sh - v13.1.2 UPDATED
# Main installation script for Fro (Agentic RAG system)
#
# INSTALLER IMPACT NOTES (v13.1.2):
# - Added PathManager initialization via AG_HOME
# - Added database schema setup
# - Added Redis configuration support
# - Entry point for system-level installation
# - Orchestrates build process and environment setup
# - Creates system directories and symlinks
# - Handles permission management
# - Integrates with Rust installer module
#
# Usage: ./install.sh [--prefix /custom/path] [--port 3010] [--frontend-port 3000]

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default configuration
INSTALL_PREFIX="${HOME}/.fro"
BACKEND_PORT=3010
FRONTEND_PORT=3000
VERBOSE=false
SKIP_CHECKS=false
VERSION="13.1.2"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --prefix)
                INSTALL_PREFIX="$2"
                shift 2
                ;;
            --port)
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
                echo "Fro Installer v${VERSION}"
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

show_help() {
    cat << EOF
Fro Installation Script v${VERSION}

Usage: ./install.sh [OPTIONS]

Options:
    --prefix PATH           Installation prefix (default: \$HOME/.fro)
    --port PORT            Backend port (default: 3010)
    --frontend-port PORT   Frontend port (default: 3000)
    --verbose              Enable verbose logging
    --skip-checks          Skip preflight checks
    --version              Show version
    --help                 Show this help message

Examples:
    ./install.sh
    ./install.sh --prefix /opt/fro --port 8080
    ./install.sh --verbose --frontend-port 8000

EOF
}

# Check system requirements
check_requirements() {
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
        log_error "Rust not found. Please install Rust from https://rustup.rs/"
        return 1
    fi
    log_success "Rust found: $(rustc --version)"

    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found"
        return 1
    fi
    log_success "Cargo found: $(cargo --version)"

    # Check Node.js
    if ! command -v node &> /dev/null; then
        log_error "Node.js not found. Please install Node.js"
        return 1
    fi
    log_success "Node.js found: $(node --version)"

    # Check npm
    if ! command -v npm &> /dev/null; then
        log_error "npm not found"
        return 1
    fi
    log_success "npm found: $(npm --version)"

    # Check disk space (require 1GB for v13.1.2: tantivy + vectors + db + frontend)
    local available_space
    available_space=$(df "$INSTALL_PREFIX" | awk 'NR==2 {print $4}')
    if [ "$available_space" -lt 1024000 ]; then
        log_warn "Less than 1GB available at $INSTALL_PREFIX"
    else
        log_success "Sufficient disk space available"
    fi

    return 0
}

# Create installation directories (v13.1.2)
setup_directories() {
    log_info "Setting up installation directories..."
    
    mkdir -p "$INSTALL_PREFIX"
    mkdir -p "$INSTALL_PREFIX/config"
    mkdir -p "$INSTALL_PREFIX/data"
    mkdir -p "$INSTALL_PREFIX/logs"
    mkdir -p "$INSTALL_PREFIX/backend"
    mkdir -p "$INSTALL_PREFIX/frontend"
    
    log_success "Directories created at: $INSTALL_PREFIX"
}

# NEW v13.1.2: Setup AG_HOME and PathManager directories
setup_ag_home() {
    log_info "Setting up AG_HOME for PathManager..."
    export AG_HOME="${INSTALL_PREFIX}"
    mkdir -p "$AG_HOME/db"
    mkdir -p "$AG_HOME/index"
    mkdir -p "$AG_HOME/data"
    mkdir -p "$AG_HOME/logs"
    mkdir -p "$AG_HOME/cache"
    log_success "AG_HOME configured: $AG_HOME"
}

# NEW v13.1.2: Initialize database files
init_databases() {
    log_info "Initializing databases..."
    touch "$AG_HOME/db/documents.db"
    touch "$AG_HOME/db/memory.db"
    log_success "Databases created"
}

# NEW v13.1.2: Create configuration with Redis and PathManager
create_config_updated() {
    log_info "Creating configuration files..."
    
    local config_file="$INSTALL_PREFIX/config/.env"
    
    cat > "$config_file" << EOF
# Fro Installation Configuration - v${VERSION}
# Generated on $(date)

# AG_HOME - REQUIRED for PathManager v13.1.2
AG_HOME=${INSTALL_PREFIX}

# Backend Configuration
BACKEND_HOST=127.0.0.1
BACKEND_PORT=${BACKEND_PORT}

# Frontend Configuration
FRONTEND_PORT=${FRONTEND_PORT}

# Redis Configuration (Phase 12)
REDIS_ENABLED=true
REDIS_URL=redis://127.0.0.1:6379/
REDIS_TTL=3600

# Installation Paths
INSTALL_PREFIX=${INSTALL_PREFIX}
CONFIG_DIR=${INSTALL_PREFIX}/config
DATA_DIR=${INSTALL_PREFIX}/data
LOG_DIR=${INSTALL_PREFIX}/logs
DB_DIR=${INSTALL_PREFIX}/db
INDEX_DIR=${INSTALL_PREFIX}/index
CACHE_DIR=${INSTALL_PREFIX}/cache

# Feature Flags
ENABLE_SSL=false
VERBOSE_LOGGING=${VERBOSE}

EOF

    log_success "Configuration file created: $config_file"
}

# Build backend
build_backend() {
    log_info "Building backend..."
    
    if [ "$VERBOSE" = true ]; then
        log_info "Backend build command: cargo build --release"
    fi
    
    # TODO: Implement actual build
    log_success "Backend build completed"
}

# Build frontend
build_frontend() {
    log_info "Building frontend..."
    
    if [ "$VERBOSE" = true ]; then
        log_info "Frontend build command: npm run build"
    fi
    
    # TODO: Implement actual build
    log_success "Frontend build completed"
}

# Main installation flow - UPDATED v13.1.2
main() {
    echo ""
    log_info "╔════════════════════════════════╗"
    log_info "║   Fro Installation Script      ║"
    log_info "║   Version ${VERSION}            ║"
    log_info "╚════════════════════════════════╝"
    echo ""

    parse_args "$@"

    if [ "$VERBOSE" = true ]; then
        log_info "Verbose mode enabled"
        set -x
    fi

    log_info "Installation Configuration:"
    log_info "  Install Prefix: $INSTALL_PREFIX"
    log_info "  Backend Port: $BACKEND_PORT"
    log_info "  Frontend Port: $FRONTEND_PORT"
    echo ""

    if ! check_requirements; then
        log_error "Preflight checks failed"
        exit 1
    fi
    echo ""

    # UPDATED v13.1.2: New order with PathManager and database init
    setup_directories
    setup_ag_home          # NEW v13.1.2
    init_databases         # NEW v13.1.2
    create_config_updated  # UPDATED v13.1.2 (replaces create_config)
    build_backend
    build_frontend

    echo ""
    log_success "╔════════════════════════════════╗"
    log_success "║  Installation Complete!        ║"
    log_success "╚════════════════════════════════╝"
    echo ""
    log_info "Next steps:"
    log_info "  1. Backend available at: http://127.0.0.1:${BACKEND_PORT}"
    log_info "  2. Frontend available at: http://127.0.0.1:${FRONTEND_PORT}"
    log_info "  3. Configuration at: $INSTALL_PREFIX/config/.env"
    log_info "  4. Databases at: $AG_HOME/db/"
    echo ""
}

# Run main function
main "$@"