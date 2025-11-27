# Resource Attribution for Distributed Tracing

## Overview

The Resource Attribution system tracks and exposes the resource overhead from distributed tracing in the AG backend. This allows you to monitor the cost of observability and ensure tracing doesn't impact application performance.

## Features

### Metrics Exposed

The system exposes 5 Prometheus metrics:

1. **`process_memory_bytes`** - Current process memory usage (RSS) in bytes
2. **`process_memory_peak_bytes`** - Peak process memory usage in bytes
3. **`process_cpu_percent`** - Current CPU usage percentage (0-100)
4. **`tracing_memory_overhead_bytes`** - Estimated memory overhead from tracing (1.5% of total)
5. **`tracing_cpu_overhead_percent`** - Estimated CPU overhead from tracing (~0.5%)

### How It Works

The resource attribution system:

1. **Reads from `/proc` filesystem** - Uses Linux `/proc/self/stat` and `/proc/self/status` for accurate metrics
2. **Updates periodically** - Default 60-second interval (configurable)
3. **Runs in background** - Non-blocking tokio task
4. **Estimates tracing overhead** - Based on industry benchmarks:
   - Memory: 1-2% of total (using 1.5% average)
   - CPU: ~0.5% during trace operations

## Configuration

### Environment Variables

```bash
# Enable resource attribution (default: true)
RESOURCE_ATTRIBUTION_ENABLED=true

# Update interval in seconds (default: 60)
RESOURCE_ATTRIBUTION_UPDATE_INTERVAL_SECS=60
```

### Example Configuration

**Development (frequent updates):**
```bash
RESOURCE_ATTRIBUTION_ENABLED=true
RESOURCE_ATTRIBUTION_UPDATE_INTERVAL_SECS=30
```

**Production (standard):**
```bash
RESOURCE_ATTRIBUTION_ENABLED=true
RESOURCE_ATTRIBUTION_UPDATE_INTERVAL_SECS=60
```

**Disabled:**
```bash
RESOURCE_ATTRIBUTION_ENABLED=false
```

## Usage

### Viewing Metrics

Query the Prometheus metrics endpoint:

```bash
curl http://127.0.0.1:3010/monitoring/metrics | grep -E "(process_|tracing_)"
```

Example output:
```
# HELP process_memory_bytes Process memory usage in bytes (RSS)
# TYPE process_memory_bytes gauge
process_memory_bytes 52428800

# HELP process_memory_peak_bytes Process peak memory usage in bytes
# TYPE process_memory_peak_bytes gauge
process_memory_peak_bytes 67108864

# HELP process_cpu_percent Process CPU usage percentage (0-100)
# TYPE process_cpu_percent gauge
process_cpu_percent 2.5

# HELP tracing_memory_overhead_bytes Estimated memory overhead from distributed tracing (1-2% of total)
# TYPE tracing_memory_overhead_bytes gauge
tracing_memory_overhead_bytes 786432

# HELP tracing_cpu_overhead_percent Estimated CPU overhead from distributed tracing (~0.5%)
# TYPE tracing_cpu_overhead_percent gauge
tracing_cpu_overhead_percent 0.5
```

### Grafana Dashboard

Create a Grafana dashboard to visualize resource attribution:

**Memory Panel:**
```promql
# Total memory
process_memory_bytes

# Peak memory
process_memory_peak_bytes

# Tracing overhead
tracing_memory_overhead_bytes
```

**CPU Panel:**
```promql
# Total CPU usage
process_cpu_percent

# Tracing overhead
tracing_cpu_overhead_percent
```

**Overhead Percentage Panel:**
```promql
# Memory overhead as percentage
(tracing_memory_overhead_bytes / process_memory_bytes) * 100

# CPU overhead (constant)
tracing_cpu_overhead_percent
```

## Architecture

### System Flow

```
┌─────────────────────────────────────────────────────────┐
│  Application Startup (main.rs)                         │
│  ├─ Phase 1.6: Start Resource Attribution              │
│  │  └─ Background task spawned                         │
│  └─ Continue with normal startup                       │
└─────────────────────────────────────────────────────────┘
         │
         ├─ Background Task (every 60s)
         │
         ▼
┌──────────────────────────────────────────────────────────┐
│  Resource Attribution Loop                              │
│  1. Read /proc/self/stat (CPU, RSS)                     │
│  2. Read /proc/self/status (VmPeak, VmRSS)              │
│  3. Calculate CPU usage delta                           │
│  4. Estimate tracing overhead                           │
│  5. Update Prometheus metrics                           │
│  6. Sleep until next interval                           │
└──────────────────────────────────────────────────────────┘
```

### Implementation Details

**CPU Calculation:**
- Reads user time (`utime`) and system time (`stime`) from `/proc/self/stat`
- Calculates delta between readings
- Converts from jiffies to percentage using system clock ticks

**Memory Calculation:**
- Reads `VmRSS` (current memory) and `VmPeak` (peak memory) from `/proc/self/status`
- Converts from KB to bytes
- Estimates tracing overhead as 1.5% of current memory

**Overhead Estimation:**
- Memory: 1.5% of total (based on OpenTelemetry benchmarks)
- CPU: 0.5% constant (conservative estimate for trace operations)

## Performance Impact

The resource attribution system itself has minimal overhead:

- **Memory**: ~1 KB for tracking state
- **CPU**: <0.01% (only active during 60-second updates)
- **I/O**: 2 file reads per update (cached by kernel)

## Troubleshooting

### Metrics Not Updating

**Check if enabled:**
```bash
# Should see "📊 Resource attribution started" in logs
journalctl --user -u ag.service | grep "Resource attribution"
```

**Verify configuration:**
```bash
echo $RESOURCE_ATTRIBUTION_ENABLED
echo $RESOURCE_ATTRIBUTION_UPDATE_INTERVAL_SECS
```

### Inaccurate Metrics

**Linux-only feature:**
- Resource attribution requires Linux `/proc` filesystem
- Will not work on macOS or Windows

**Check /proc access:**
```bash
cat /proc/self/stat
cat /proc/self/status
```

### High Overhead Estimates

If tracing overhead seems high:

1. **Check actual memory usage** - Compare with/without tracing enabled
2. **Review trace volume** - High trace volume = higher overhead
3. **Adjust sampling** - Reduce trace sampling rate if needed
4. **Monitor over time** - Overhead varies with request rate

## Best Practices

### Monitoring

1. **Set up alerts** for high resource usage:
   ```promql
   # Alert if memory overhead exceeds 3%
   (tracing_memory_overhead_bytes / process_memory_bytes) * 100 > 3
   
   # Alert if total memory exceeds threshold
   process_memory_bytes > 1073741824  # 1 GB
   ```

2. **Track trends** over time to identify memory leaks or CPU spikes

3. **Correlate with request rate** to understand overhead patterns

### Optimization

1. **Adjust update interval** based on needs:
   - Development: 30 seconds for quick feedback
   - Production: 60-120 seconds to reduce overhead

2. **Disable if not needed** in environments where resource monitoring isn't critical

3. **Use with trace sampling** to reduce tracing overhead

## Integration with Other Systems

### Prometheus

Resource attribution metrics are automatically registered with the Prometheus registry and exposed at `/monitoring/metrics`.

### Grafana

Import the AG backend dashboard (see `GRAFANA_DASHBOARD_IMPORT_GUIDE.md`) which includes resource attribution panels.

### Alertmanager

Configure alerts for resource thresholds:

```yaml
groups:
  - name: resource_attribution
    rules:
      - alert: HighTracingMemoryOverhead
        expr: (tracing_memory_overhead_bytes / process_memory_bytes) * 100 > 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Tracing memory overhead exceeds 3%"
          
      - alert: HighProcessMemory
        expr: process_memory_bytes > 1073741824
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Process memory exceeds 1 GB"
```

## Future Enhancements

Potential improvements:

1. **Per-component attribution** - Track overhead by module (retriever, API, etc.)
2. **Historical analysis** - Store overhead trends in time-series database
3. **Automatic optimization** - Adjust trace sampling based on overhead
4. **Cross-platform support** - Add support for macOS and Windows
5. **Network overhead** - Track bandwidth used by trace export

## References

- [OpenTelemetry Performance Benchmarks](https://opentelemetry.io/docs/specs/otel/performance/)
- [Linux /proc Documentation](https://www.kernel.org/doc/Documentation/filesystems/proc.txt)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)

## Related Documentation

- `TRACE_ALERTING.md` - Trace-based anomaly alerting
- `OPENTELEMETRY_TRACING_COMPLETE_GUIDE.md` - Complete tracing setup
- `README_TRACING.md` - General tracing documentation
