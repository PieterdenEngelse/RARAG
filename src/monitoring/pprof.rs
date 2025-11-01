//! pprof stub endpoints (Phase 15 Step 3)
//! Optional dev-only profiling endpoints behind `profiling` feature.
//! Default build returns 501 Not Implemented.

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde_json::json;

#[cfg(feature = "profiling")]
fn not_implemented_msg() -> &'static str { "profiling feature enabled; implementation pending" }

#[cfg(feature = "profiling")]
fn dummy_metadata(kind: &str) -> serde_json::Value {
    serde_json::json!({
        "enabled": true,
        "kind": kind,
        "note": "profiling integration forthcoming"
    })
}
#[cfg(not(feature = "profiling"))]
fn not_implemented_msg() -> &'static str { "profiling disabled; build with --features profiling" }

/// GET /monitoring/pprof/cpu
pub async fn pprof_cpu() -> ActixResult<HttpResponse> {
    let body = json!({
        "status": "not_implemented",
        "message": not_implemented_msg(),
        "endpoint": "/monitoring/pprof/cpu"
    });
    #[cfg(feature = "profiling")]
    {
        body["metadata"] = dummy_metadata("cpu");
    }
    Ok(HttpResponse::NotImplemented().json(body))
}

/// GET /monitoring/pprof/heap
pub async fn pprof_heap() -> ActixResult<HttpResponse> {
    let body = json!({
        "status": "not_implemented",
        "message": not_implemented_msg(),
        "endpoint": "/monitoring/pprof/heap"
    });
    #[cfg(feature = "profiling")]
    {
        body["metadata"] = dummy_metadata("heap");
    }
    Ok(HttpResponse::NotImplemented().json(body))
}

/// Register pprof routes under /monitoring/pprof
pub fn register_pprof_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/monitoring/pprof")
            .route("/cpu", web::get().to(pprof_cpu))
            .route("/heap", web::get().to(pprof_heap))
    );
}
