use alvio_observe::{get_metrics, health_handler, metrics_handler, ready_handler};
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use http_body_util::BodyExt;
use std::sync::atomic::Ordering;
use tower::ServiceExt;

#[tokio::test]
async fn test_prometheus_metrics_endpoint() {
    let metrics = get_metrics();
    metrics.record_packet_in(1450);
    metrics.record_packet_out(1450);
    metrics.rooms_active.store(8, Ordering::Relaxed);
    metrics.peers_active.store(24, Ordering::Relaxed);

    let app = Router::new().route("/metrics", get(metrics_handler));

    let req = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let ct = resp
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(ct.contains("text/plain"));

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body.to_vec()).unwrap();

    assert!(body_str.contains("# HELP alvio_packets_in_total"));
    assert!(body_str.contains("# TYPE alvio_packets_in_total counter"));
    assert!(body_str.contains("alvio_rooms_active 8"));
    assert!(body_str.contains("alvio_peers_active 24"));
}

#[tokio::test]
async fn test_health_and_ready_endpoints() {
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler));

    // 1. Health check
    let health_req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let health_resp = app.clone().oneshot(health_req).await.unwrap();
    assert_eq!(health_resp.status(), StatusCode::OK);

    let health_body = health_resp.into_body().collect().await.unwrap().to_bytes();
    let health_json: serde_json::Value = serde_json::from_slice(&health_body).unwrap();
    assert_eq!(health_json["status"], "healthy");
    assert!(health_json["version"].is_string());

    // 2. Ready check
    let ready_req = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let ready_resp = app.clone().oneshot(ready_req).await.unwrap();
    assert_eq!(ready_resp.status(), StatusCode::OK);

    let ready_body = ready_resp.into_body().collect().await.unwrap().to_bytes();
    let ready_json: serde_json::Value = serde_json::from_slice(&ready_body).unwrap();
    assert_eq!(ready_json["status"], "ready");

    // 3. Drain Mode returns 503 Service Unavailable
    alvio_observe::set_draining(true);
    let draining_req = Request::builder()
        .uri("/ready")
        .body(Body::empty())
        .unwrap();

    let draining_resp = app.oneshot(draining_req).await.unwrap();
    assert_eq!(draining_resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    alvio_observe::set_draining(false);
}
