use std::future::{ready, Ready};
use std::task::{Context, Poll};
use std::time::Instant;
use actix_service::{Service, Transform};
use actix_web::{dev::{ServiceRequest, ServiceResponse}, Error, http::header};
use tracing::{info_span, Instrument};
use crate::monitoring::metrics::REQUEST_LATENCY_MS;

pub struct TraceMiddleware;

impl TraceMiddleware {
    pub fn new() -> Self { Self }
}

impl<S, B> Transform<S, ServiceRequest> for TraceMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TraceMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> <Self as Transform<S, ServiceRequest>>::Future {
        ready(Ok(TraceMiddlewareService { service }))
    }
}

pub struct TraceMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TraceMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), <Self as Service<ServiceRequest>>::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> <Self as Service<ServiceRequest>>::Future {
        let method = req.method().to_string();
        // Normalize route label: use matched resource pattern/name when available to avoid high cardinality
        let route_label = req.match_pattern().unwrap_or_else(|| req.path().to_string());
        let user_agent = req.headers().get(header::USER_AGENT).and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let request_id = uuid::Uuid::new_v4().to_string();

        // Try to capture client IP from connection info; middleware runs before our helper
        let client_ip = req.connection_info().realip_remote_addr().unwrap_or("unknown").to_string();

        let span = info_span!("http_request", method = %method, path = %route_label, client_ip = %client_ip, request_id = %request_id, user_agent = %user_agent);
        let start = Instant::now();
        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.instrument(span).await;
            if let Ok(ref response) = res {
                let status = response.status().as_u16();
                let duration_ms = start.elapsed().as_millis() as u64;
                let status_class = format!("{}xx", status / 100);
                // Record to Prometheus histogram
                REQUEST_LATENCY_MS
                    .with_label_values(&[&method, &route_label, &status_class])
                    .observe(duration_ms as f64);
                tracing::info!(method = %method, path = %route_label, status = status, duration_ms = duration_ms, request_id = %request_id, "request completed");
            }
            res
        })
    }
}
