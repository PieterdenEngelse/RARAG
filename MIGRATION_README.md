# Promtail → Vector Migration

## Quick Start

You're currently using **Promtail** but should be using **Vector**. This migration will:

1. ✅ Preserve all log sources (AG, monitoring, OS logs)
2. ✅ Keep all labels and error tracking
3. ✅ Improve performance
4. ✅ Clean up Promtail

## Run Migration

```bash
# 1. Copy enhanced Vector config
cp ./vector_enhanced.toml ~/.config/vector/vector.toml

# 2. Validate config
~/.local/bin/vector validate --config ~/.config/vector/vector.toml

# 3. Run migration script
./migrate_back_to_vector.sh
```

The script is **interactive** and will:
- Stop Promtail
- Start Vector
- Verify logs are flowing
- Optionally clean up Promtail files

## Files Created

| File | Purpose |
|------|---------|
| `vector_enhanced.toml` | Enhanced Vector config with all log sources |
| `migrate_back_to_vector.sh` | Interactive migration script |
| `VECTOR_MIGRATION_GUIDE.md` | Detailed migration documentation |
| `MIGRATION_README.md` | This file |

## What Gets Migrated

**7 Log Sources:**
1. `systemd-journal` - AG service logs
2. `ag-file-logs` - AG file logs
3. `systemd-monitoring` - Loki, OTEL, Prometheus, Grafana, Alertmanager
4. `system-errors` - Critical systemd errors
5. `kernel` - Kernel/hardware errors
6. `auth` - Authentication/security events
7. `syslog` - General system logs

**All Labels Preserved:**
- `level`, `request_id`, `http_status`, `http_method`, `duration_ms`
- `is_error`, `is_warning`
- `systemd_unit`, `hostname`, `priority`
- `log_type`, `auth_event`, `process`

## Verification

After migration:

```bash
# Check Vector is running
systemctl --user status vector.service

# Check logs are in Loki
curl -s "http://127.0.0.1:3100/loki/api/v1/label/job/values"

# Should show all 7 jobs:
# ["ag-file-logs","auth","kernel","syslog","system-errors","systemd-journal","systemd-monitoring"]
```

## Cleanup

The migration script will optionally remove:
- `~/.local/bin/promtail`
- `~/.config/promtail/config.yml`
- `~/.config/systemd/user/promtail.service`
- `~/.local/share/promtail/`

**Backups are preserved** in `~/.config/promtail/backup/`

## Need Help?

See `VECTOR_MIGRATION_GUIDE.md` for:
- Detailed troubleshooting
- Configuration mapping
- Performance comparison
- Rollback instructions

## Summary

**Before:** Promtail → Loki (7 log sources)
**After:** Vector → Loki (same 7 log sources, better performance)

**No changes to:**
- Loki queries
- Grafana dashboards
- Log labels
- Error tracking

**Benefits:**
- ✅ Better performance
- ✅ More powerful transformations (VRL)
- ✅ Built-in metrics (port 9598)
- ✅ Better error handling
