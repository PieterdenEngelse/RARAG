#!/bin/bash

# ==============================================================================
# Alertmanager Installation Script - Fixed Version
# Version: 1.0.1
# Date: 2025-11-21
# Usage: bash alertmanager_install_fixed.sh
# ==============================================================================

set -e

# Colors for output (used only in echo statements, never in file paths)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
ALERTMANAGER_VERSION="0.26.0"
INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/alertmanager"
DATA_DIR="$HOME/.local/share/alertmanager"
SERVICE_DIR="$HOME/.config/systemd/user"

# ==============================================================================
# Functions
# ==============================================================================

log_info() {
  echo -e "${BLUE}ℹ ${NC}$1"
}

log_success() {
  echo -e "${GREEN}✓ ${NC}$1"
}

log_warning() {
  echo -e "${YELLOW}⚠ ${NC}$1"
}

log_error() {
  echo -e "${RED}✗ ${NC}$1"
}

check_prerequisites() {
  log_info "Checking prerequisites..."
  
  # Check if running on Linux
  if [[ ! "$OSTYPE" == "linux"* ]]; then
    log_error "This script only works on Linux"
    exit 1
  fi
  log_success "Linux system detected"
  
  # Check internet connectivity
  if ! ping -c 1 github.com &> /dev/null; then
    log_warning "Cannot reach github.com - will use local binaries if available"
  else
    log_success "Internet connectivity OK"
  fi
  
  # Check disk space
  available_space=$(df "$HOME" | tail -1 | awk '{print $4}')
  if [ "$available_space" -lt 100000 ]; then
    log_error "Less than 100MB free disk space"
    exit 1
  fi
  log_success "Disk space available: ${available_space}KB"
  
  # Check port 9093
  if lsof -i :9093 &> /dev/null 2>&1; then
    log_error "Port 9093 is already in use"
    exit 1
  fi
  log_success "Port 9093 is available"
}

detect_architecture() {
  log_info "Detecting system architecture..."
  
  ARCH=$(uname -m)
  case $ARCH in
    x86_64)
      ALERTMANAGER_ARCH="amd64"
      ;;
    aarch64)
      ALERTMANAGER_ARCH="arm64"
      ;;
    armv7l)
      ALERTMANAGER_ARCH="armv7"
      ;;
    *)
      log_error "Unsupported architecture: $ARCH"
      exit 1
      ;;
  esac
  
  log_success "Architecture: $ARCH (Alertmanager: $ALERTMANAGER_ARCH)"
}

download_alertmanager() {
  log_info "Downloading Alertmanager $ALERTMANAGER_VERSION..."
  
  TEMP_DIR=$(mktemp -d)
  cd "$TEMP_DIR"
  
  URL="https://github.com/prometheus/alertmanager/releases/download/v${ALERTMANAGER_VERSION}/alertmanager-${ALERTMANAGER_VERSION}.linux-${ALERTMANAGER_ARCH}.tar.gz"
  
  if ! wget -q "$URL" -O alertmanager.tar.gz; then
    log_error "Failed to download from $URL"
    rm -rf "$TEMP_DIR"
    exit 1
  fi
  
  log_success "Downloaded successfully"
  
  # Extract
  log_info "Extracting archive..."
  tar xzf alertmanager.tar.gz
  
  # Find extracted directory name
  EXTRACTED_DIR=$(find . -maxdepth 1 -type d -name "alertmanager-*" | head -1)
  if [ -z "$EXTRACTED_DIR" ]; then
    log_error "Failed to extract archive"
    rm -rf "$TEMP_DIR"
    exit 1
  fi
  
  log_success "Extracted to temporary location"
  
  echo "$TEMP_DIR"
}

install_binaries() {
  TEMP_DIR="$1"
  
  log_info "Installing binaries to $INSTALL_DIR..."
  
  # Create install directory
  mkdir -p "$INSTALL_DIR"
  
  # Find alertmanager binary in extracted directory
  EXTRACTED_DIR=$(find "$TEMP_DIR" -maxdepth 1 -type d -name "alertmanager-*" | head -1)
  
  if [ ! -f "$EXTRACTED_DIR/alertmanager" ]; then
    log_error "Alertmanager binary not found in archive"
    exit 1
  fi
  
  # Install alertmanager
  cp "$EXTRACTED_DIR/alertmanager" "$INSTALL_DIR/alertmanager"
  chmod +x "$INSTALL_DIR/alertmanager"
  
  log_success "Alertmanager binary installed"
  
  # Install amtool if available
  if [ -f "$EXTRACTED_DIR/amtool" ]; then
    cp "$EXTRACTED_DIR/amtool" "$INSTALL_DIR/amtool"
    chmod +x "$INSTALL_DIR/amtool"
    log_success "Amtool binary installed"
  fi
  
  # Verify
  if ! "$INSTALL_DIR/alertmanager" --version &> /dev/null; then
    log_error "Binary verification failed"
    exit 1
  fi
  
  log_success "Binary verification passed"
  
  # Show version
  VERSION=$("$INSTALL_DIR/alertmanager" --version 2>&1 | head -1)
  log_info "Installed: $VERSION"
}

create_directories() {
  log_info "Creating configuration directories..."
  
  mkdir -p "$CONFIG_DIR"
  mkdir -p "$DATA_DIR"
  mkdir -p "$SERVICE_DIR"
  
  chmod 700 "$CONFIG_DIR"
  chmod 700 "$DATA_DIR"
  
  log_success "Directories created and permissions set"
}

create_config_file() {
  log_info "Creating default configuration file..."
  
  if [ -f "$CONFIG_DIR/alertmanager.yml" ]; then
    log_warning "Configuration file already exists at $CONFIG_DIR/alertmanager.yml"
    log_info "Skipping configuration file creation (backup existing first if needed)"
    return
  fi
  
  cat > "$CONFIG_DIR/alertmanager.yml" << 'EOF'
# Default Alertmanager Configuration
# Update this file with your specific settings

global:
  resolve_timeout: 5m

route:
  receiver: 'default'
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 12h

receivers:
  - name: 'default'

inhibit_rules: []
EOF

  chmod 600 "$CONFIG_DIR/alertmanager.yml"
  
  log_success "Configuration file created"
  log_info "Edit configuration at: $CONFIG_DIR/alertmanager.yml"
}

validate_config() {
  log_info "Validating configuration..."
  
  if ! "$INSTALL_DIR/alertmanager" \
    --config.file="$CONFIG_DIR/alertmanager.yml" \
    --check-config &> /dev/null; then
    log_error "Configuration validation failed"
    exit 1
  fi
  
  log_success "Configuration is valid"
}

create_systemd_service() {
  log_info "Creating systemd service file..."
  
  cat > "$SERVICE_DIR/alertmanager.service" << 'EOF'
[Unit]
Description=Prometheus Alertmanager
Documentation=https://prometheus.io/docs/alerting/latest/overview/
Requires=network-online.target
After=network-online.target
PartOf=ag-observability.target

[Service]
Type=simple
User=%u
ExecStart=%h/.local/bin/alertmanager \
  --config.file=%h/.config/alertmanager/alertmanager.yml \
  --storage.path=%h/.local/share/alertmanager/ \
  --web.listen-address=127.0.0.1:9093

Restart=on-failure
RestartSec=5s
MemoryMax=500M
CPUQuota=50%
StandardOutput=journal
StandardError=journal
SyslogIdentifier=alertmanager

[Install]
WantedBy=default.target
EOF

  chmod 644 "$SERVICE_DIR/alertmanager.service"
  
  log_success "Systemd service file created"
}

enable_and_start_service() {
  log_info "Enabling and starting service..."
  
  # Reload systemd
  systemctl --user daemon-reload
  log_success "Systemd daemon reloaded"
  
  # Enable service
  systemctl --user enable alertmanager.service
  log_success "Service enabled for autostart"
  
  # Start service
  systemctl --user start alertmanager.service
  sleep 2
  
  log_success "Service started"
}

verify_installation() {
  log_info "Verifying installation..."
  
  # Check if binary is in PATH
  if ! command -v alertmanager &> /dev/null; then
    log_warning "Alertmanager is not in PATH"
    log_info "Add to PATH: export PATH=\$PATH:$INSTALL_DIR"
  else
    log_success "Alertmanager found in PATH"
  fi
  
  # Check service status
  if ! systemctl --user is-active alertmanager.service > /dev/null; then
    log_error "Service is not running"
    log_info "Check logs: journalctl --user-unit=alertmanager.service -n 20"
    exit 1
  fi
  log_success "Service is running"
  
  # Check port
  sleep 1
  if ! lsof -i :9093 &> /dev/null 2>&1; then
    log_error "Port 9093 is not listening"
    exit 1
  fi
  log_success "Port 9093 is listening"
  
  # Health check
  if ! curl -s http://127.0.0.1:9093/-/healthy 2>&1 | grep -q "healthy"; then
    log_error "Health check failed"
    exit 1
  fi
  log_success "Health check passed"
  
  # API check
  if ! curl -s http://127.0.0.1:9093/api/v1/status 2>&1 | jq . > /dev/null 2>&1; then
    log_error "API check failed"
    exit 1
  fi
  log_success "API check passed"
}

cleanup() {
  log_info "Cleaning up temporary files..."
  
  if [ -d "$1" ]; then
    rm -rf "$1"
    log_success "Temporary files removed"
  fi
}

show_summary() {
  echo ""
  echo -e "${GREEN}========================================${NC}"
  echo -e "${GREEN}Alertmanager Installation Complete ✓${NC}"
  echo -e "${GREEN}========================================${NC}"
  echo ""
  
  echo "Binary Location: $INSTALL_DIR/alertmanager"
  echo "Configuration:   $CONFIG_DIR/alertmanager.yml"
  echo "Data Directory:  $DATA_DIR"
  echo "Service File:    $SERVICE_DIR/alertmanager.service"
  echo ""
  
  echo "Service Status:"
  systemctl --user status alertmanager.service --no-pager | head -5
  echo ""
  
  echo "Web UI:          http://127.0.0.1:9093"
  echo "Health Check:    http://127.0.0.1:9093/-/healthy"
  echo "API Status:      http://127.0.0.1:9093/api/v1/status"
  echo ""
  
  echo "Useful Commands:"
  echo "  View Logs:       journalctl --user-unit=alertmanager.service -f"
  echo "  Stop Service:    systemctl --user stop alertmanager.service"
  echo "  Start Service:   systemctl --user start alertmanager.service"
  echo "  Edit Config:     nano $CONFIG_DIR/alertmanager.yml"
  echo ""
  
  echo "Next Steps:"
  echo "  1. Configure receivers in alertmanager.yml"
  echo "  2. Connect Prometheus (see Phase 17 integration guide)"
  echo "  3. Deploy alert rules (phase17-rules.yml)"
  echo "  4. Test alerts by triggering a test condition"
  echo ""
}

# ==============================================================================
# Main Installation Flow
# ==============================================================================

main() {
  echo -e "${BLUE}"
  echo "========================================"
  echo "Alertmanager Installation Script"
  echo "========================================"
  echo -e "${NC}"
  echo ""
  
  check_prerequisites
  echo ""
  
  detect_architecture
  echo ""
  
  TEMP_DIR=$(download_alertmanager)
  echo ""
  
  install_binaries "$TEMP_DIR"
  echo ""
  
  create_directories
  echo ""
  
  create_config_file
  echo ""
  
  validate_config
  echo ""
  
  create_systemd_service
  echo ""
  
  enable_and_start_service
  echo ""
  
  verify_installation
  echo ""
  
  cleanup "$TEMP_DIR"
  echo ""
  
  show_summary
}

# ==============================================================================
# Error Handling
# ==============================================================================

trap 'log_error "Installation failed"; exit 1' ERR

# ==============================================================================
# Run Installation
# ==============================================================================

main "$@"