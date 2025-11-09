Here’s a ready-to-import Grafana dashboard JSON (CPU, memory, filesystem, uptime). Copy all and paste into Grafana: Dashboards → New → Import → “Import via dashboard JSON” → Select your Prometheus data source when prompted.

"graphMode": "none", "justifyMode": "auto", "orientation": "auto", "reduceOptions": { "calcs": ["lastNotNull"], "fields": "", "values": false }, "textMode": "auto" }, "targets": [ { "refId": "A", "expr": "(time() - node_boot_time_seconds) / 86400", "legendFormat": "{{instance}}", "instant": true } ] } ], "refresh": "30s", "schemaVersion": 39, "style": "dark", "tags": ["node", "system", "basic"], "templating": { "list": [] }, "time": { "from": "now-1h", "to": "now" }, "timezone": "", "title": "Node Overview (Local)", "version": 1, "weekStart": "" }

Import steps:

    Grafana → Dashboards → New → Import
    Paste the JSON above → Continue
    Select your Prometheus data source → Import

Want me to import it automatically via the Grafana API? If so, provide an API key with admin permission (or user/pass), and I’ll POST it to http://127.0.0.1:1789/api/dashboards/db for you.

