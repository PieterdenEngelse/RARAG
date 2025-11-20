#!/usr/bin/env bash
#
# OpenTelemetry Collector Installer
# Phase 17: Distributed Tracing Infrastructure
# Version: 1.0
#
# Installs OTel Collector Contrib (includes logging exporter)
# Supports:
#  - Local user-space setup (for development/testing)
#  - Systemd service setup (requires sudo, for production)
#
# Usage:
#  chmod +x install_otelcol.sh
#  ./install_otelcol.sh [--mode user|system] [--version 0.88.0]
#

set -euo pipefail

# Configuration
COLLECTOR_VERSION="${2:-0.88.0}"
INSTALL_MODE="${1:-user}"  # user or system
ARCH="linux_amd64"
RELEASE_URL="https://github.com/open-telemetry/opentelemetry-collector-releases/releases/download"

# Directories
if [ "$INSTALL_MODE" = "user" ]; then
    INSTALL_DIR="$HOME/.local/bin"
    CONFIG_DIR="$HOME/.config/otelcol"
    DATA_DIR="$HOME/.local/share/otelcol"
    SERVICE_USER="$USER"
else
    INSTALL_DIR="/usr/local/bin"
    CONFIG_DIR="/etc/otelcol"
    DATA_DIR="/var/lib/otelcol"
    SERVICE_USER="otelcol"
fi

LOG_DIR="$DATA_DIR/logs"
BINARY_NAME="otelcontribcol"
BINARY_PATH="${INSTALL_DIR}/${BINARY_NAME}"
CONFIG_FILE="${CONFIG_DIR}/config.yaml"
SYSTEMD_FILE="/etc/systemd/system/otelcol.service"
DOWNLOAD_URL="${RELEASE_URL}/v${COLLECTOR_VERSION}/otelcontribcol_${COLLECTOR_VERSION}_${ARCH}.tar.gz"
TEMP_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "════════════════════════════════════════════════════════════"
echo "OpenTelemetry Collector Installer"
echo "════════════════════════════════════════════════════════════"
echo "Version: ${COLLECTOR_VERSION}"
echo "Mode: ${INSTALL_MODE}"
echo "Install Dir: ${INSTALL_DIR}"
echo "Config Dir: ${CONFIG_DIR}"
echo "Data Dir: ${DATA_DIR}"
echo ""

# Check if already installed
if [ -f "$BINARY_PATH" ]; then
    echo "⚠️  OTel Collector already installed at $BINARY_PATH"
    $BINARY_PATH version
    read -p "Overwrite? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborting."
        exit 0
    fi
fi

# Create directories
echo "Creating directories..."
mkdir -p "$INSTALL_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$DATA_DIR"
mkdir -p "$LOG_DIR"
chmod 755 "$CONFIG_DIR" "$DATA_DIR" "$LOG_DIR"
echo "✓ Directories created"
echo ""

# Download binary
echo "Downloading OTel Collector ${COLLECTOR_VERSION}..."
echo "URL: $DOWNLOAD_URL"
cd "$TEMP_DIR"
if ! curl -fsSL "$DOWNLOAD_URL" -o otelcol.tar.gz; then
    echo "❌ Download failed. Check version number and internet connection."
    exit 1
fi
echo "✓ Downloaded"
echo ""

# Extract
echo "Extracting..."
tar xzf otelcol.tar.gz
if [ ! -f "otelcontribcol" ]; then
    echo "❌ Extraction failed - binary not found"
    exit 1
fi
chmod +x otelcontribcol
echo "✓ Extracted"
echo ""

# Copy to install directory
echo "Installing binary to ${INSTALL_DIR}..."
cp otelcontribcol "$BINARY_PATH"
chmod +x "$BINARY_PATH"
echo "✓ Installed"
echo ""

# Verify installation
echo "Verifying installation..."
$BINARY_PATH version
echo "✓ Verification successful"
echo ""

# Create configuration
echo "Creating configuration..."
cat > "$CONFIG_FILE" << 'EOF'
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4317
      http:
        endpoint: 127.0.0.1:4318

exporters:
  logging:
    loglevel: info
  otlp:
    client:
      endpoint: localhost:4317
      tls:
        insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [logging]
    metrics:
      receivers: [otlp]
      exporters: [logging]

extensions:
  health_check:
    endpoint: 127.0.0.1:13133
EOF

chmod 644 "$CONFIG_FILE"
echo "✓ Configuration created: $CONFIG_FILE"
echo ""

# Create systemd service (if system mode)
if [ "$INSTALL_MODE" = "system" ]; then
    echo "Creating systemd service..."
    if [ ! -w /etc/systemd/system ]; then
        echo "⚠️  Need sudo to create systemd service"
        sudo tee "$SYSTEMD_FILE" > /dev/null << EOF
[Unit]
Description=OpenTelemetry Collector
After=network.target
Documentation=https://opentelemetry.io/docs/collector/

[Service]
Type=notify
User=$SERVICE_USER
Group=$SERVICE_USER
ExecStart=$BINARY_PATH --config=$CONFIG_FILE
ExecReload=/bin/kill -HUP \$MAINPID
Restart=on-failure
RestartSec=5s

# Performance tuning
StandardOutput=journal
StandardError=journal
SyslogIdentifier=otelcol

[Install]
WantedBy=multi-user.target
EOF
        sudo systemctl daemon-reload
        echo "✓ Systemd service created: $SYSTEMD_FILE"
        echo ""
        echo "To start the service:"
        echo "  sudo systemctl start otelcol"
        echo "  sudo systemctl enable otelcol  # auto-start on boot"
        echo "  sudo systemctl status otelcol"
    else
        tee "$SYSTEMD_FILE" > /dev/null << EOF
[Unit]
Description=OpenTelemetry Collector
After=network.target
Documentation=https://opentelemetry.io/docs/collector/

[Service]
Type=simple
User=$SERVICE_USER
ExecStart=$BINARY_PATH --config=$CONFIG_FILE
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
EOF
        systemctl --user daemon-reload 2>/dev/null || true
        echo "✓ User systemd service created"
    fi
    echo ""
fi

# Create startup script for user mode
if [ "$INSTALL_MODE" = "user" ]; then
    STARTUP_SCRIPT="${CONFIG_DIR}/start-otelcol.sh"
    cat > "$STARTUP_SCRIPT" << EOF
#!/usr/bin/env bash
# Start OTel Collector in user mode
export OTEL_RESOURCE_ATTRIBUTES="service.name=agentic-rag,service.version=13.1.2"
$BINARY_PATH --config=$CONFIG_FILE
EOF
    chmod +x "$STARTUP_SCRIPT"
    echo "✓ Startup script created: $STARTUP_SCRIPT"
    echo ""
fi

# Summary
echo "════════════════════════════════════════════════════════════"
echo "✅ Installation Complete!"
echo "════════════════════════════════════════════════════════════"
echo ""
echo "📦 Binary: $BINARY_PATH"
echo "📄 Config: $CONFIG_FILE"
echo "📊 Logs: $LOG_DIR"
echo ""

if [ "$INSTALL_MODE" = "user" ]; then
    echo "🚀 To start the collector (development mode):"
    echo ""
    echo "   $STARTUP_SCRIPT"
    echo ""
    echo "   Or manually:"
    echo "   $BINARY_PATH --config=$CONFIG_FILE"
    echo ""
    echo "   Listens on:"
    echo "     • gRPC OTLP:  127.0.0.1:4317"
    echo "     • HTTP OTLP:  127.0.0.1:4318"
    echo "     • Health:     127.0.0.1:13133"
    echo ""
else
    echo "🚀 To start the collector (service mode):"
    echo ""
    echo "   sudo systemctl start otelcol"
    echo "   sudo systemctl enable otelcol  # auto-start on boot"
    echo ""
fi

echo "🔗 Next Steps:"
echo ""
echo "1. Start the collector:"
if [ "$INSTALL_MODE" = "user" ]; then
    echo "   $STARTUP_SCRIPT"
else
    echo "   sudo systemctl start otelcol"
fi
echo ""
echo "2. Switch your app to OTLP mode:"
echo "   export OTEL_OTLP_EXPORT=true"
echo "   export OTEL_CONSOLE_EXPORT=false"
echo "   export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317"
echo "   cargo run"
echo ""
echo "3. Test trace propagation:"
echo "   bash /mnt/user-data/outputs/test_tracing_otlp_v1.0.sh"
echo ""
echo "4. Monitor collector output:"
echo "   tail -f ${LOG_DIR}/otelcol.log"
echo ""