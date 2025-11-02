use actix_web::{test, App};
use ag::monitoring::handlers;

#[actix_web::test]
async fn cpu_returns_501_by_default() {
    let app = test::init_service(
        App::new().configure(|cfg| {
            // reuse monitoring handlers registration (Option A wiring)
            handlers::register_routes(cfg);
        })
    ).await;

    let req = test::TestRequest::get()
        .uri("/monitoring/pprof/cpu")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_IMPLEMENTED);
}

#[actix_web::test]
async fn heap_returns_501_by_default() {
    let app = test::init_service(
        App::new().configure(|cfg| {
            handlers::register_routes(cfg);
        })
    ).await;

    let req = test::TestRequest::get()
        .uri("/monitoring/pprof/heap")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_IMPLEMENTED);
}
