# Migration Guide: Promtail → Vector

## Overview

This guide helps you migrate from Promtail back to Vector while preserving all the enhanced log collection capabilities we set up (AG service, monitoring stack, OS logs).

## Why Vector?

- **Better performance** - More efficient resource usage
- **Richer transformations** - VRL (Vector Remap Language) is more powerful
- **Built-in metrics** - Prometheus exporter included
- **Single binary** - No external dependencies
- **Better error handling** - More robust retry logic

## Migration Steps

### Step 1: Copy Enhanced Vector Configuration

```bash
# Copy the enhanced configuration
cp ./vector_enhanced.toml ~/.config/vector/vector.toml

# Verify the configuration
~/.local/bin/vector validate --config ~/.config/vector/vector.toml
```

### Step 2: Run Migration Script

```bash
# Make the script executable
chmod +x ./migrate_back_to_vector.sh

# Run the migration
./migrate_back_to_vector.sh
```

The script will:
1. ✓ Stop and disable Promtail
2. ✓ Verify Vector binary exists
3. ✓ Create Vector systemd service
4. ✓ Start Vector
5. ✓ Verify logs are flowing to Loki
6. ✓ Optionally clean up Promtail files

### Step 3: Verify Migration

**Check Vector is running:**
```bash
systemctl --user status vector.service
```

**Check Vector logs:**
```bash
journalctl --user -u vector.service -f
```

**Check Vector metrics:**
```bash
curl http://localhost:9598/metrics
```

**Verify logs in Loki:**
```bash
# Check available jobs
curl -s "http://127.0.0.1:3100/loki/api/v1/label/job/values"

# Should return:
# ["ag-file-logs","auth","kernel","syslog","system-errors","systemd-journal","systemd-monitoring"]
```

## Log Sources Comparison

| Source | Promtail | Vector | Status |
|--------|----------|--------|--------|
| AG journald | ✅ | ✅ | Migrated |
| AG file logs | ✅ | ✅ | Migrated |
| Monitoring stack | ✅ | ✅ | Migrated |
| System errors | ✅ | ✅ | Migrated |
| Kernel logs | ✅ | ✅ | Migrated |
| Auth logs | ✅ | ✅ | Migrated |
| Syslog | ✅ | ✅ | Migrated |

## Configuration Mapping

### Promtail → Vector Equivalents

**Promtail pipeline stages:**
```yaml
pipeline_stages:
  - regex:
      expression: '.*level=(?P<level>info|warn|error)'
  - labels:
      level:
```

**Vector transforms:**
```toml
[transforms.extract_level]
type = "remap"
source = '''
  level_match = parse_regex(.message, r'level=(info|warn|error)') ?? {}
  if exists(level_match.1) {
    .level = level_match.1
  }
'''
```

### Label Extraction

**Promtail:**
- Uses `relabel_configs` and `pipeline_stages`
- Limited regex capabilities
- Static labels via `static_labels`

**Vector:**
- Uses VRL (Vector Remap Language)
- Full regex support with `parse_regex()`
- Dynamic label assignment
- Conditional logic with `if/else`

## Files and Directories

### Vector Files (Keep)

```
~/.local/bin/vector                    # Binary
~/.config/vector/vector.toml           # Configuration
~/.config/vector/vector.toml.backup    # Backup
~/.config/systemd/user/vector.service  # Systemd service
~/.local/share/vector/                 # Data directory
```

### Promtail Files (Remove After Migration)

```
~/.local/bin/promtail                         # Binary
~/.config/promtail/config.yml                 # Configuration
~/.config/promtail/backup/                    # Backups (keep)
~/.config/systemd/user/promtail.service       # Systemd service
~/.local/share/promtail/                      # Data directory
```

## Cleanup Commands

If you didn't run cleanup during migration:

```bash
# Stop and disable Promtail
systemctl --user stop promtail.service
systemctl --user disable promtail.service

# Remove Promtail files
rm ~/.local/bin/promtail
rm ~/.config/promtail/config.yml
rm ~/.config/systemd/user/promtail.service
rm -rf ~/.local/share/promtail

# Reload systemd
systemctl --user daemon-reload
```

**Keep backups:**
```bash
# These are safe to keep for reference
~/.config/promtail/backup/
~/.config/vector/vector.toml.backup
```

## Troubleshooting

### Vector Won't Start

**Check configuration syntax:**
```bash
~/.local/bin/vector validate --config ~/.config/vector/vector.toml
```

**Check logs:**
```bash
journalctl --user -u vector.service -n 50
```

**Common issues:**
- Syntax errors in TOML
- Missing data directory
- Port conflicts (9598 for metrics)

### No Logs in Loki

**Check Vector is sending:**
```bash
# Vector metrics show sent events
curl -s http://localhost:9598/metrics | grep loki_events_sent
```

**Check Loki is receiving:**
```bash
curl -s "http://127.0.0.1:3100/loki/api/v1/query" \
  --data-urlencode 'query={job=~".*"}' \
  --data-urlencode 'limit=1'
```

**Check Vector logs for errors:**
```bash
journalctl --user -u vector.service | grep -i error
```

### Labels Not Appearing

**Verify transform logic:**
```bash
# Test VRL expressions
~/.local/bin/vector vrl
```

**Check label cardinality:**
```bash
# Too many unique labels can cause issues
curl -s "http://127.0.0.1:3100/loki/api/v1/labels"
```

## Performance Comparison

### Resource Usage

**Promtail:**
- Memory: ~25-30MB
- CPU: Low, spikes during log bursts

**Vector:**
- Memory: ~40-50MB (more features)
- CPU: More efficient, better batching
- Disk: Configurable buffers

### Throughput

**Vector advantages:**
- Better batching (configurable)
- Async processing
- Built-in backpressure handling
- Disk buffering for reliability

## Query Examples (Same for Both)

```bash
# All errors
{is_error="true"}

# AG service logs
{job="systemd-journal"}

# OS errors
{job=~"kernel|auth|syslog|system-errors", is_error="true"}

# Specific service
{systemd_unit="loki.service"}
```

## Rollback (If Needed)

If you need to go back to Promtail:

```bash
# Stop Vector
systemctl --user stop vector.service
systemctl --user disable vector.service

# Restore Promtail config from backup
cp ~/.config/promtail/backup/config.yml.* ~/.config/promtail/config.yml

# Start Promtail
systemctl --user start promtail.service
systemctl --user enable promtail.service
```

## Summary

**Migration checklist:**
- ✅ Vector configuration created
- ✅ Vector service running
- ✅ Logs flowing to Loki
- ✅ All 7 log sources working
- ✅ Labels extracted correctly
- ✅ Promtail stopped
- ✅ Promtail files cleaned up

**Post-migration:**
- Monitor Vector metrics: `http://localhost:9598/metrics`
- Check Vector logs: `journalctl --user -u vector.service -f`
- Verify Loki queries work as before

**Benefits achieved:**
- ✅ Better performance
- ✅ More powerful transformations
- ✅ Built-in metrics
- ✅ Same log coverage
- ✅ Same query capabilities
