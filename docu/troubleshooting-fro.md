requests:

When request rate spikes alongside rising latency/errors, you can investigate the upstream traffic or dependencies with these steps:

1. **Confirm backend load**  
   - Check the Requests tab’s chart plus Overview tab to see if the spike is sustained.  
   - Use `curl http://127.0.0.1:3010/monitoring/metrics | grep http_requests_total` to see if certain routes are receiving unusual traffic; look at labels like `method`, `status`, or `route`.

2. **Trace the source of traffic**  
   - Examine ingress/load-balancer logs (e.g., Nginx/Envoy) to identify client IPs or services sending the surge.  
   - If you have synthetic traffic scripts (like `run_sample_traffic`), temporarily disable them to ensure they aren’t the culprit.

3. **Inspect service dependencies**  
   - Use the Cache, Index, or Rate Limit tabs to see if downstream components look saturated (low hit rate, rate-limit drops).  
   - Check any external services (Redis, vector DB, embedding providers). For example, run `redis-cli INFO` or consult its own monitoring panel.

4. **Peek at logs for rollback hints**  
   - From the Logs tab, filter for WARN/ERROR around the spike timeframe.  
   - On the server, tail logs directly: `journalctl -u ag.service -n 200 -f` or `tail -f logs/ag.log`. Look for connection timeouts, dependency failures, or unusually slow handlers.

5. **Use tracing / metrics dashboards**  
   - If OpenTelemetry is enabled, view traces in Tempo/Grafana to see which spans accumulate latency (e.g., DB queries, embedding generation).  
   - In Grafana/Prometheus, plot latency histograms (`http_request_duration_ms_bucket`) or DB pool metrics to spot bottlenecks.

6. **Engage upstream teams**  
   - If the load originates from another internal service, coordinate with its owners. Suggest rate limiting or backoff if it’s misbehaving.

Once you identify the root cause (e.g., high client traffic, dependency slowness, network failures), you can right-size resources, add caching, or throttle the offending traffic.

Cash: more info:

                                             
                                                                                                    
    1. **Open both panels side-by-side**                                                            
                                                                                                    
                                                                                                    
        * In the frontend, keep the Cache tab open with the checklist expanded.                     
                                                                                                    
        * In another browser tab or window, open the Requests tab and expand its troubleshooting    
checklist.                                                                                          
    2. **Capture the cache snapshot**                                                               
                                                                                                    
                                                                                                    
        * In a terminal on the same host that’s running `ag`, run:      curl                        
http://127.0.0.1:3010/monitor/cache/info                                                            
                                                                                                    
             Save the output (e.g., redirect to a file) so you can reference exact hit/miss counts, 
TTL, and Redis status.                                                                              
    3. **Collect traces/metrics at the same moment**                                                
                                                                                                    
                                                                                                    
        * While the cache info is captured, open your Tempo/Grafana dashboards and note the relevant
 traces—especially around the timeframe when misses spike.                                          
                                                                                                    
        * Screenshot or export the trace view and any Prometheus panels (Request Rate/Volume,       
`cache_misses_total`, latency histograms, etc.) so you can correlate them later.                    
    4. **Compare notes**                                                                            
                                                                                                    
                                                                                                    
        * Use the Cache checklist to interpret what you see in the `cache/info` output.       

        Use the Cache checklist to interpret what you see in the `cache/info` output.             
                                                                                                    
        * Use the Requests checklist to diagnose whether the load spike aligns with those cache     
misses.                                                                                             
    5. **Document findings**                                                                        
                                                                                                    
                                                                                                    
        * Record the `curl` output, screenshots, and any trace links in your troubleshooting notes  
or ticket so it’s easy to share with teammates.                                                     
                                                                                                    
Once you’ve gathered both the cache snapshot and the trace/metric context, you’ll have              
the “paired” evidence the checklist is nudging you toward. 

2. Message + JSON

      * Keep the current JSON snapshot, but add a short textual `message` and maybe a few key
  fields (file, mode) as separate tracing fields.
      * Example log call:    tracing::info!(
                target = "chunking_snapshot",
                message = "chunking snapshot recorded",
                file = %snapshot.file,
                mode = %snapshot.chunker_mode,
                snapshot = %serde_json::to_string(&snapshot).unwrap_or_default(),
            );
      * Benefit: when tailing logs you’ll immediately see the message and filename, but the full
  JSON is still there if you need it.

  ### 3. Log-level filtering

      * Because we’re using a dedicated target (`chunking_snapshot`), you can control it via
  `RUST_LOG`.
          * To keep it at `info` (default): `RUST_LOG=info,chunking_snapshot=info`

          * To silence it: `RUST_LOG=info,chunking_snapshot=warn` (or exclude it altogether)

          * To see it only when debugging: set our log call to `debug!` and run with
  `chunking_snapshot=debug`.
      * If you want a runtime toggle, we could add an env var like `CHUNKING_SNAPSHOT_LOG_LEVEL`,
   but often using `RUST_LOG` is enough.

  Would you like me to implement the message+JSON logging now, or are you just
  evaluating the approach?

### User-controlled logging (Step 3 enhancement) ✅

  To let end users toggle snapshot logging at runtime:

      1. **Config & Env**


          * Added `chunking_log_enabled` to `ApiConfig` and documented
  `CHUNKING_SNAPSHOT_LOGGING` in `.env.example`.

          * Default is `true`; set the env var to `false`/`0` to disable logging.
      2. **Runtime toggle**


          * `main.rs` now calls
  `ag::monitoring::set_chunking_logging_enabled(config.chunking_log_enabled)` during startup.

          * `record_chunking_snapshot` checks a shared atomic flag before emitting the log. When
  logging is enabled, we still write the JSON snapshot to the `chunking_snapshot` target; when
  disabled, only the in-memory buffer is updated.
      3. **Public API**


          * Exposed `set_chunking_logging_enabled` from the monitoring module so other subsystems
   (or future admin endpoints) can flip it if needed.

  Tests: cargo fmt, cargo check.

  Now operators can control logging without code changes—just set the env var (or call
  the helper if you later expose it via an admin API). Let me know if you’d also like
  a /monitoring/chunking/logging?enabled=false endpoint for hot toggles.


Now operators can control logging without code changes—just set the env var (or call
  the helper if you later expose it via an admin API). Let me know if you’d also like
  a /monitoring/chunking/logging?enabled=false endpoint for hot toggles.

### Hot-toggle endpoint added for chunking snapshot logging ✅                                 
                                                                                                 
  You liked the idea of letting operators flip logging without restarts, so we wired             
  it up:                                                                                         

      1. **New endpoint**


          * `GET /monitoring/chunking/logging`

              * `?enabled=true` or `?enabled=false` updates the runtime flag immediately.


              * No query param returns the current status.


              * Example:    curl
  "http://localhost:3010/monitoring/chunking/logging?enabled=false"
                         curl "http://localhost:3010/monitoring/chunking/logging"
      2. **Runtime state**


          * `set_chunking_logging_enabled` stores the flag in an atomic `LOGGING_ENABLED`.

          * Added `chunking_logging_enabled()` to report the current setting.
      3. **Config integration**


          * Startup still honors `CHUNKING_SNAPSHOT_LOGGING` (default true) via `ApiConfig` →
  `main.rs`.
      4. **Docs**


          * `.env.example` now lists the env var for clarity.

  Tests: cargo fmt, cargo check.

  Now operators can script or manually toggle snapshot logging on a live system
  without touching environment variables or restarting the service. Let me know if
  you’d like the endpoint to require auth or move under a different path.

 Next up per the roadmap: Phase 4 Task 3.3 – integrate the reranker (pending start).

