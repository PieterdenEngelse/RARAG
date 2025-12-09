OptimizationExpected GainImplementation EffortMemory Impact

Better embeddings+10-25%LowVaries

Reranking+15-30%MediumLow-Medium

Query expansion+10-20%LowMinimal

Hybrid search+10-15%MediumLow

Lightweight adaptive chunking+2-5%MediumMinimal

Given the current use case (just wanting an adjustable history for inspection),
  logging every snapshot is the most logical first step: minimal code, no new
  infrastructure, survives restarts. If you later find yourself wanting to plot
  trends, add a handful of targeted Prometheus gauges (e.g., semantic_flushes_total)
  rather than stuffing full snapshots into metrics.

 Got it—you were responding to the note about log formatting and filtering. Here are
  the concrete options in case you do want adjustments:

      1. **Flatten the log fields:**


          * Instead of storing the whole snapshot as a JSON string, we can log each field (file,
  mode, threshold, etc.) as separate tracing fields.

          * Pros: easier to query in Loki/ELK without parsing the JSON string.

          * Cons: log entries get wider; you’d want to keep the number of fields manageable.
