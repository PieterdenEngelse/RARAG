//! HTTP handlers for monitoring endpoints
//! 
//! Endpoints:
//! - GET /monitoring/health - Full health status (JSON)
//! - GET /monitoring/ready - Readiness probe (K8s compatible)
//! - GET /monitoring/live - Liveness probe (K8s compatible)
//! - GET /monitoring/metrics - Prometheus format metrics

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;
use super::MonitoringContext;

/// Health check endpoint
/// 
/// Returns full health status including component details
/// 
/// Response:
/// ```json
/// {
///   "status": "healthy",
///   "timestamp": "2025-10-26T12:30:45Z",
///   "uptime_seconds": 123.45,
///   "components": {
///     "api": "healthy",
///     "database": "healthy",
///     "configuration": "healthy",
///     "logging": "healthy"
///   }
/// }
/// ```
pub async fn health_handler(
    ctx: web::Data<MonitoringContext>,
) -> ActixResult<HttpResponse> {
    let status = ctx.health_status();
    
    let status_code = match status.status {
        super::health::ComponentStatus::Healthy => actix_web::http::StatusCode::OK,
        super::health::ComponentStatus::Degraded => actix_web::http::StatusCode::OK,
        super::health::ComponentStatus::Unhealthy => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
    };
    
    Ok(HttpResponse::build(status_code).json(status))
}

/// Readiness probe endpoint (K8s compatible)
/// 
/// Returns 200 if system is ready to accept traffic
/// Returns 503 if not ready
pub async fn ready_handler(
    ctx: web::Data<MonitoringContext>,
) -> ActixResult<HttpResponse> {
    if ctx.health.is_ready() {
        Ok(HttpResponse::Ok().json(json!({
            "ready": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "ready": false,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })))
    }
}

/// Liveness probe endpoint (K8s compatible)
/// 
/// Returns 200 if process is alive
/// Returns 503 if process should be restarted
pub async fn live_handler(
    ctx: web::Data<MonitoringContext>,
) -> ActixResult<HttpResponse> {
    if ctx.health.is_live() {
        Ok(HttpResponse::Ok().json(json!({
            "live": true,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })))
    } else {
        Ok(HttpResponse::ServiceUnavailable().json(json!({
            "live": false,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })))
    }
}

/// Metrics endpoint (Prometheus format)
/// 
/// Returns metrics in Prometheus text format
/// Content-Type: text/plain; version=0.0.4
pub async fn metrics_handler(
    ctx: web::Data<MonitoringContext>,
) -> ActixResult<HttpResponse> {
    match ctx.metrics.export() {
        Ok(metrics_text) => {
            Ok(HttpResponse::Ok()
                .content_type("text/plain; version=0.0.4; charset=utf-8")
                .body(metrics_text))
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to export metrics");
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": "Failed to export metrics"
            })))
        }
    }
}

/// Register monitoring routes
/// 
/// INSTALLER IMPACT:
/// - These routes must be registered in main API setup
/// - Should be registered early before other routes
pub fn register_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/monitoring")
            .route("/health", web::get().to(health_handler))
            .route("/ready", web::get().to(ready_handler))
            .route("/live", web::get().to(live_handler))
            .route("/metrics", web::get().to(metrics_handler))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;
    
    #[actix_web::test]
    async fn test_health_endpoint() {
        let config = super::super::config::MonitoringConfig::default();
        let ctx = super::super::MonitoringContext::new(config)
            .expect("Failed to create context");
        let ctx = web::Data::new(ctx);
        
        let _req = test::TestRequest::get()
            .uri("/health")
            .to_http_request();
        
        let resp = health_handler(ctx).await.unwrap();
        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    }
}