# Loki Query Performance Analysis

## Query: `{is_error="true"}` - All Errors Across All Sources

### ⚡ Performance Metrics (Current System)

**Test Results:**

| Query | Bytes Processed | Lines Processed | Execution Time | Total Time |
|-------|----------------|-----------------|----------------|------------|
| `{is_error="true"}` | 391 KB | 306 lines | 0.018s | 0.033s |
| `{job="system-errors", is_error="true"}` | 369 KB | 291 lines | 0.008s | 0.022s |

**Key Findings:**
- ✅ **Very fast:** < 20ms execution time
- ✅ **Low data volume:** ~400 KB processed
- ✅ **Efficient:** Only 306 log lines scanned
- ⚠️ **2x slower** than specific job query (but still fast)

---

## 📊 Performance Impact Analysis

### **Current Impact: MINIMAL** ✅

**Why it's efficient:**

1. **Label-based filtering is fast**
   - Loki uses inverted indexes for labels
   - `is_error="true"` is a label, not a text search
   - No need to scan log content

2. **Small data volume**
   - Only ~400 KB processed per query
   - Only 306 error lines in the time range
   - Most logs are NOT errors (good!)

3. **Fast execution**
   - 18ms is very fast for a cross-source query
   - Grafana auto-refresh at 30s is fine

---

## 🔍 Comparison: Label vs Text Search

### **Label Filter (Fast):**
```
{is_error="true"}                    ← 18ms, uses index
{job="system-errors"}                ← 8ms, uses index
{systemd_unit="ag.service"}          ← Fast, uses index
```

### **Text Search (Slower):**
```
{job="systemd-journal"} |= "error"   ← Slower, scans content
{job="systemd-journal"} |~ "err.*"   ← Slowest, regex on content
```

**Performance difference:**
- Label queries: **Index lookup** (milliseconds)
- Text searches: **Full scan** (seconds for large datasets)

---

## 📈 Scalability Projections

### **Current Scale:**
- 306 error lines in default time range
- 391 KB data processed
- 18ms query time

### **At 10x Scale (3,000 errors):**
- Estimated: ~3.9 MB data
- Estimated: ~50-100ms query time
- **Still acceptable** for dashboards

### **At 100x Scale (30,000 errors):**
- Estimated: ~39 MB data
- Estimated: ~500ms-1s query time
- **May need optimization:**
  - Add time range limits
  - Use specific job filters
  - Implement query caching

---

## 💡 Optimization Strategies

### **1. Add Time Range Limits (Recommended)**

Instead of:
```
{is_error="true"}
```

Use:
```
{is_error="true"} [5m]    ← Last 5 minutes only
```

**Impact:** Reduces data scanned by limiting time window

---

### **2. Use More Specific Queries**

**Good (specific):**
```
{job="system-errors", is_error="true"}           ← 8ms
{job="systemd-journal", is_error="true"}         ← Fast
```

**Less optimal (broad):**
```
{is_error="true"}                                ← 18ms (still fast)
```

**Difference:** 2x faster with job filter

---

### **3. Use Metric Queries for Dashboards**

For time-series panels, use aggregations:

```
sum(rate({is_error="true"}[5m]))                 ← Very fast
sum by (job) (rate({is_error="true"}[5m]))       ← Fast, grouped
```

**Why faster:**
- Pre-aggregated data
- Less data transferred
- Better for graphs

---

### **4. Implement Query Caching**

Grafana caches query results automatically:
- Same query within cache TTL = instant
- Default TTL: 5 minutes
- No additional config needed

---

## 🎯 Recommendations

### **For Your Current Setup: ✅ NO CHANGES NEEDED**

**Reasons:**
1. Query is already fast (18ms)
2. Data volume is small (391 KB)
3. Error rate is low (306 errors)
4. System has plenty of capacity

### **When to Optimize:**

**Optimize if:**
- Query time > 1 second
- Data processed > 100 MB per query
- Dashboard feels slow
- Error volume increases 100x

**How to optimize:**
1. Add `[5m]` time range to queries
2. Use `sum(rate(...))` for metrics
3. Split into separate panels by job
4. Increase Grafana cache TTL

---

## 📊 Query Performance Tiers

### **Tier 1: Instant (< 50ms)** ✅ ← You are here
- Single job queries
- Label-only filters
- Small time ranges

### **Tier 2: Fast (50-500ms)**
- Multi-job queries
- Moderate time ranges
- Simple text filters

### **Tier 3: Acceptable (500ms-2s)**
- Cross-source queries
- Large time ranges
- Complex regex

### **Tier 4: Slow (> 2s)**
- Full-text search across all logs
- Very large time ranges
- Complex aggregations

---

## 🔧 Monitoring Query Performance

### **In Grafana:**

1. **Check query inspector:**
   - Edit panel → Query inspector
   - Shows execution time and data processed

2. **Watch for slow queries:**
   - Query time > 1s = investigate
   - Data processed > 100 MB = optimize

3. **Use Loki metrics:**
   - Query Loki's own metrics at `:3100/metrics`
   - Monitor `loki_request_duration_seconds`

---

## 📝 Best Practices

### **DO:**
✅ Use label filters (`{job="x", is_error="true"}`)
✅ Limit time ranges (`[5m]`, `[1h]`)
✅ Use metric queries for graphs (`rate()`, `sum()`)
✅ Cache frequently used queries
✅ Monitor query performance

### **DON'T:**
❌ Use broad text searches (`{} |= "error"`)
❌ Query unlimited time ranges
❌ Use complex regex on large datasets
❌ Run expensive queries on auto-refresh
❌ Ignore slow query warnings

---

## 🎯 Conclusion

**Query: `{is_error="true"}`**

**Performance Rating: ⭐⭐⭐⭐⭐ EXCELLENT**

- ✅ Fast execution (18ms)
- ✅ Low resource usage (391 KB)
- ✅ Scalable to 10x current volume
- ✅ No optimization needed now
- ✅ Safe for auto-refresh dashboards

**Recommendation:** **Use it!** The query is efficient and well-suited for your use case.

---

## 📚 Additional Resources

- **Loki Query Performance:** https://grafana.com/docs/loki/latest/operations/query-performance/
- **LogQL Optimization:** https://grafana.com/docs/loki/latest/logql/
- **Grafana Caching:** https://grafana.com/docs/grafana/latest/administration/data-source-management/

---

**Last Updated:** 2025-11-24
**System:** Vector → Loki → Grafana
**Query Tested:** `{is_error="true"}`
