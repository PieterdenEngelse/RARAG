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


