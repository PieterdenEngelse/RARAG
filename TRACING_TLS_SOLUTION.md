# Distributed Tracing TLS Configuration - Final Solution

## Problem Summary

The AG backend distributed tracing setup is **99% complete** but traces are not reaching Tempo due to a TLS configuration mismatch:

1. **AG Backend** → Sends traces via gRPC to OpenTelemetry Collector on port 4318 ✅
2. **OpenTelemetry Collector** → Tries to forward to Tempo on port 4317 with TLS ❌
3. **Tempo** → Listening on port 4317 with TLS (self-signed certificate)

**The Issue**: The OpenTelemetry Collector's OTLP exporter with `tls.insecure_skip_verify: true` is trying to connect to port 9095 (default TLS port) instead of port 4317.

## Root Cause

When the OpenTelemetry Collector OTLP exporter sees:
```yaml
exporters:
  otlp/tempo:
    endpoint: 127.0.0.1:4317
    tls:
      insecure_skip_verify: true
```

It interprets this as "use TLS with the default port (9095)" instead of "use TLS on port 4317".

## Solution Options

### Option 1: Configure Tempo to Accept Plaintext gRPC (Recommended for Development)

Modify Tempo's configuration to accept plaintext gRPC on a different port:

```bash
# Edit Tempo configuration
sudo nano /etc/tempo/config.yml

# Add a plaintext gRPC receiver:
distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"  # Keep TLS version
          tls:
            cert_file: /etc/tempo/tls/tempo.crt
            key_file: /etc/tempo/tls/tempo.key
        grpc_plaintext:  # Add plaintext version
          endpoint: "127.0.0.1:4320"

# Restart Tempo
sudo systemctl restart tempo

# Update OpenTelemetry Collector config
nano ~/.config/otelcol/config.yaml

# Change exporter to:
exporters:
  otlp/tempo:
    endpoint: 127.0.0.1:4320  # Use plaintext port
    tls:
      insecure: true  # Disable TLS

# Restart OpenTelemetry Collector
systemctl --user restart otelcol.service
```

### Option 2: Fix OpenTelemetry Collector TLS Configuration

Explicitly configure the OTLP exporter to use TLS on port 4317:

```yaml
exporters:
  otlp/tempo:
    endpoint: 127.0.0.1:4317
    tls:
      insecure_skip_verify: true
      # Force TLS on this specific port
      insecure: false
```

However, this may still not work due to how the OTLP exporter interprets the configuration.

### Option 3: Use HTTP/JSON Protocol Instead of gRPC

Configure Tempo to accept HTTP/JSON OTLP on a different port:

```bash
# Edit Tempo configuration
sudo nano /etc/tempo/config.yml

# Add HTTP receiver:
distributor:
  receivers:
    otlp:
      protocols:
        grpc:
          endpoint: "0.0.0.0:4317"
          tls:
            cert_file: /etc/tempo/tls/tempo.crt
            key_file: /etc/tempo/tls/tempo.key
        http:  # Add HTTP protocol
          endpoint: "127.0.0.1:4321"

# Restart Tempo
sudo systemctl restart tempo

# Update OpenTelemetry Collector config
nano ~/.config/otelcol/config.yaml

# Change exporter to use HTTP:
exporters:
  otlphttp/tempo:
    endpoint: http://127.0.0.1:4321

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [tail_sampling, batch]
      exporters: [otlphttp/tempo]  # Use HTTP exporter

# Restart OpenTelemetry Collector
systemctl --user restart otelcol.service
```

## Recommended Implementation (Option 1)

The simplest and most reliable solution for development:

```bash
#!/bin/bash
# File: /home/pde/ag/fix-tracing-tls.sh

echo "Fixing distributed tracing TLS configuration..."

# 1. Backup Tempo config
sudo cp /etc/tempo/config.yml /etc/tempo/config.yml.backup-before-plaintext

# 2. Add plaintext gRPC receiver to Tempo (manual step required)
echo "⚠️  Manual step required:"
echo "Edit /etc/tempo/config.yml and add under distributor.receivers.otlp.protocols:"
echo ""
echo "    grpc_plaintext:"
echo "      endpoint: \"127.0.0.1:4320\""
echo ""
echo "Then run: sudo systemctl restart tempo"
echo ""

# 3. Update OpenTelemetry Collector config
cat > ~/.config/otelcol/config.yaml << 'OTEL_EOF'
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 127.0.0.1:4318
      http:
        endpoint: 127.0.0.1:4319

processors:
  batch:
    send_batch_size: 512
    timeout: 5s
  tail_sampling:
    decision_wait: 2s
    policies:
      - name: errors
        type: status_code
        status_code:
          status_codes: [ERROR]
      - name: slow
        type: latency
        latency:
          threshold_ms: 500
      - name: sample_some
        type: probabilistic
        probabilistic:
          sampling_percentage: 10

exporters:
  otlp/tempo:
    endpoint: 127.0.0.1:4320  # Plaintext gRPC port
    tls:
      insecure: true  # Disable TLS
  logging:
    loglevel: error

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [tail_sampling, batch]
      exporters: [otlp/tempo]
OTEL_EOF

echo "✓ OpenTelemetry Collector config updated"
echo ""
echo "After adding the plaintext receiver to Tempo, run:"
echo "  systemctl --user restart otelcol.service"
echo "  bash /home/pde/ag/verify-tracing.sh"
