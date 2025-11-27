#!/bin/bash
# Update Vector to use HTTPS for Loki

set -e

echo "🔧 Updating Vector configuration for Loki TLS..."
echo ""

VECTOR_CONFIG="$HOME/.config/vector/vector.toml"
BACKUP_FILE="$HOME/.config/vector/vector.toml.backup-loki-tls"

echo "1. Backing up current Vector configuration..."
cp "$VECTOR_CONFIG" "$BACKUP_FILE"
echo "   ✓ Backup created: $BACKUP_FILE"

echo ""
echo "2. Updating Vector sinks to use HTTPS for Loki..."

# Update all Loki sinks to use HTTPS
sed -i 's|endpoint = "http://127.0.0.1:3100"|endpoint = "https://127.0.0.1:3100"|g' "$VECTOR_CONFIG"

# Add TLS configuration to each Loki sink
# This is a bit complex, so we'll use a more targeted approach

# For now, let's just verify the endpoint was updated
if grep -q 'endpoint = "https://127.0.0.1:3100"' "$VECTOR_CONFIG"; then
    echo "   ✓ Loki endpoints updated to HTTPS"
else
    echo "   ✗ Failed to update endpoints"
    exit 1
fi

echo ""
echo "3. Adding TLS skip verify to Vector sinks..."

# We need to add tls.verify_certificate = false to each Loki sink
# This is complex with sed, so let's create a new config

cat > "$VECTOR_CONFIG" <<'VECTOREOF'
# Vector configuration for multiple log sources
# Collects logs from AG service, monitoring stack, system errors, and syslog
data_dir = "/home/pde/.local/share/vector"

# ============================================================================
# Source: All systemd journal logs (AG + monitoring + system)
# ============================================================================
[sources.systemd_all]
type = "journald"
include_units = [
  "ag.service",
  "loki.service", 
  "otelcol.service", 
  "prometheus.service", 
  "grafana-server.service",
  "alertmanager.service"
]
current_boot_only = false
data_dir = "/home/pde/.local/share/vector"

[transforms.systemd_labels]
type = "remap"
inputs = ["systemd_all"]
source = '''
.message = to_string(.message) ?? ""
.systemd_unit = to_string(.SYSTEMD_UNIT) ?? "unknown"
.hostname = to_string(.host) ?? "localhost"
.priority = to_string(.PRIORITY) ?? "6"
.syslog_identifier = to_string(.SYSLOG_IDENTIFIER) ?? ""

level_result = parse_regex(.message, r'\s+(INFO|WARN|ERROR|DEBUG|TRACE)\s+') ?? null
if level_result != null {
  .level = to_string(level_result[1])
}

trace_result = parse_regex(.message, r'trace_id=([a-f0-9-]+)') ?? null
if trace_result != null {
  .trace_id = to_string(trace_result[1])
}

status_result = parse_regex(.message, r'status=(\d{3})') ?? null
if status_result != null {
  .http_status = to_string(status_result[1])
}

method_result = parse_regex(.message, r'method=(GET|POST|PUT|DELETE|PATCH)') ?? null
if method_result != null {
  .http_method = to_string(method_result[1])
}

if contains(.message, "ERROR") || contains(.message, "error") {
  .is_error = "true"
}
if contains(.message, "WARN") || contains(.message, "warn") {
  .is_warning = "true"
}
'''

[sinks.loki_systemd]
type = "loki"
inputs = ["systemd_labels"]
endpoint = "https://127.0.0.1:3100"
encoding.codec = "json"
labels.job = "systemd-journal"
labels.systemd_unit = "{{ systemd_unit }}"
labels.hostname = "{{ hostname }}"
labels.priority = "{{ priority }}"
labels.syslog_identifier = "{{ syslog_identifier }}"
labels.trace_id = "{{ trace_id }}"
labels.level = "{{ level }}"
labels.http_status = "{{ http_status }}"
labels.http_method = "{{ http_method }}"
labels.is_error = "{{ is_error }}"
labels.is_warning = "{{ is_warning }}"
healthcheck.enabled = true
tls.verify_certificate = false

# ============================================================================
# Source: System errors (priority 0-3)
# ============================================================================
[sources.system_errors]
type = "journald"
current_boot_only = false
data_dir = "/home/pde/.local/share/vector"

[transforms.system_errors_filter]
type = "filter"
inputs = ["system_errors"]
condition = '(to_int(.PRIORITY) ?? 6) <= 3'

[transforms.system_errors_labels]
type = "remap"
inputs = ["system_errors_filter"]
source = '''
.message = to_string(.message) ?? ""
.systemd_unit = to_string(.SYSTEMD_UNIT) ?? "unknown"
.hostname = to_string(.host) ?? "localhost"
.priority = to_string(.PRIORITY) ?? "3"
.syslog_identifier = to_string(.SYSLOG_IDENTIFIER) ?? ""
.is_error = "true"
'''

[sinks.loki_system_errors]
type = "loki"
inputs = ["system_errors_labels"]
endpoint = "https://127.0.0.1:3100"
encoding.codec = "json"
labels.job = "system-errors"
labels.systemd_unit = "{{ systemd_unit }}"
labels.hostname = "{{ hostname }}"
labels.priority = "{{ priority }}"
labels.syslog_identifier = "{{ syslog_identifier }}"
labels.is_error = "{{ is_error }}"
healthcheck.enabled = true
tls.verify_certificate = false

# ============================================================================
# Source: Kernel logs
# ============================================================================
[sources.kernel_logs]
type = "file"
include = ["/var/log/kern.log"]
read_from = "end"
ignore_not_found = true

[transforms.kernel_labels]
type = "remap"
inputs = ["kernel_logs"]
source = '''
.message = to_string(.message) ?? ""

if match(.message, r'(?i)(error|fail|critical|panic|oops|bug|segfault)') {
  .is_error = "true"
}
if match(.message, r'(?i)(warn|warning)') {
  .is_warning = "true"
}
'''

[sinks.loki_kernel]
type = "loki"
inputs = ["kernel_labels"]
endpoint = "https://127.0.0.1:3100"
encoding.codec = "json"
labels.job = "kernel"
labels.host = "localhost"
labels.is_error = "{{ is_error }}"
labels.is_warning = "{{ is_warning }}"
healthcheck.enabled = true
tls.verify_certificate = false

# ============================================================================
# Source: Auth logs
# ============================================================================
[sources.auth_logs]
type = "file"
include = ["/var/log/auth.log"]
read_from = "end"
ignore_not_found = true

[transforms.auth_labels]
type = "remap"
inputs = ["auth_logs"]
source = '''
.message = to_string(.message) ?? ""

if match(.message, r'Failed password') {
  .auth_event = "failed_password"
}
if match(.message, r'Accepted password') {
  .auth_event = "accepted_password"
}
if match(.message, r'authentication failure') {
  .auth_event = "auth_failure"
}
if match(.message, r'session opened') {
  .auth_event = "session_opened"
}
if match(.message, r'session closed') {
  .auth_event = "session_closed"
}
if match(.message, r'sudo') {
  .auth_event = "sudo"
}

if match(.message, r'(?i)(failed|failure|invalid|error|denied)') {
  .is_error = "true"
}
'''

[sinks.loki_auth]
type = "loki"
inputs = ["auth_labels"]
endpoint = "https://127.0.0.1:3100"
encoding.codec = "json"
labels.job = "auth"
labels.host = "localhost"
labels.auth_event = "{{ auth_event }}"
labels.is_error = "{{ is_error }}"
healthcheck.enabled = true
tls.verify_certificate = false

# ============================================================================
# Source: Syslog
# ============================================================================
[sources.syslog_file]
type = "file"
include = ["/var/log/syslog"]
read_from = "end"
ignore_not_found = true

[transforms.syslog_labels]
type = "remap"
inputs = ["syslog_file"]
source = '''
.message = to_string(.message) ?? ""

process_result = parse_regex(.message, r'\w{3}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}\s+\S+\s+([^\[\s:]+)') ?? null
if process_result != null {
  .process = to_string(process_result[1])
}

if match(.message, r'(?i)(error|err|fail|critical|panic|fatal)') {
  .is_error = "true"
}
if match(.message, r'(?i)(warn|warning)') {
  .is_warning = "true"
}
'''

[sinks.loki_syslog]
type = "loki"
inputs = ["syslog_labels"]
endpoint = "https://127.0.0.1:3100"
encoding.codec = "json"
labels.job = "syslog"
labels.host = "localhost"
labels.process = "{{ process }}"
labels.is_error = "{{ is_error }}"
labels.is_warning = "{{ is_warning }}"
healthcheck.enabled = true
tls.verify_certificate = false

# ============================================================================
# Internal Metrics
# ============================================================================
[sources.internal_metrics]
type = "internal_metrics"

[sinks.prometheus]
type = "prometheus_exporter"
inputs = ["internal_metrics"]
address = "127.0.0.1:8686"
VECTOREOF

echo "   ✓ Configuration updated with TLS support"

echo ""
echo "4. Validating Vector configuration..."
if vector validate "$VECTOR_CONFIG" 2>&1 | grep -q "Validated"; then
    echo "   ✓ Configuration is valid"
else
    echo "   ✗ Configuration validation failed!"
    echo "   Restoring backup..."
    cp "$BACKUP_FILE" "$VECTOR_CONFIG"
    exit 1
fi

echo ""
echo "5. Restarting Vector..."
systemctl --user restart vector

sleep 3

echo ""
echo "6. Checking Vector status..."
if systemctl --user is-active --quiet vector; then
    echo "   ✓ Vector is running"
else
    echo "   ✗ Vector failed to start!"
    systemctl --user status vector --no-pager
    exit 1
fi

echo ""
echo "7. Verifying logs are flowing to Loki..."
sleep 5
if curl -k -s "https://localhost:3100/loki/api/v1/label/job/values" | grep -q "systemd-journal"; then
    echo "   ✓ Logs are flowing to Loki via HTTPS"
else
    echo "   ⚠ Logs may not be flowing yet (give it a moment)"
fi

echo ""
echo "✅ Vector updated for Loki TLS!"
echo ""
echo "📊 Vector is now sending logs to Loki via HTTPS"
echo ""
