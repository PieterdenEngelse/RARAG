# Observability Quick Start (Prometheus + Grafana)

This stack brings up Prometheus and Grafana with the provisioning files that live in `src/monitoring/dashboards/`.

## Prerequisites
- Docker + Docker Compose plugin.
- The backend (`ag`) running locally and exposing `/monitoring/metrics` on port 3010.

## Files
- `docker-compose.observability.yml` – launches Prometheus & Grafana.
- `ops/observability/prometheus.yml` – scrape config (edit targets as needed).
- `src/monitoring/dashboards/datasources.yaml` – Grafana datasource provisioning.
- `src/monitoring/dashboards/ag.yaml` + `src/monitoring/dashboards/ag/*.json` – dashboard provisioning files.

## Usage
```bash
# Start the stack
docker compose -f docker-compose.observability.yml up -d

# Tail logs
docker compose -f docker-compose.observability.yml logs -f

# Stop
docker compose -f docker-compose.observability.yml down
```

Prometheus → http://localhost:9090  
Grafana → http://localhost:3000 (admin/admin by default)

> **Note:** On Linux, Docker containers cannot reach the host via `host.docker.internal` by default. If your backend runs on the same host, change the target in `ops/observability/prometheus.yml` to the host IP (e.g., `host.docker.internal` on macOS/WSL, or `172.17.0.1` / actual LAN IP on native Linux).
