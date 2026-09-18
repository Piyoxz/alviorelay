use crate::metrics::get_metrics;
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static SERVER_START_TIME: OnceLock<Instant> = OnceLock::new();
static IS_DRAINING: AtomicBool = AtomicBool::new(false);

pub fn init_server_start_time() {
    SERVER_START_TIME.get_or_init(Instant::now);
}

pub fn set_draining(draining: bool) {
    IS_DRAINING.store(draining, Ordering::SeqCst);
}

pub fn is_draining() -> bool {
    IS_DRAINING.load(Ordering::Relaxed)
}

pub fn get_uptime_secs() -> u64 {
    let start = SERVER_START_TIME.get_or_init(Instant::now);
    start.elapsed().as_secs()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub uptime_secs: u64,
}

/// Endpoint handler for Prometheus `/metrics` scraper.
pub async fn metrics_handler() -> Response {
    let prometheus_text = get_metrics().render_prometheus();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );
    (StatusCode::OK, headers, prometheus_text).into_response()
}

/// Endpoint handler for Kubernetes liveness probe `/health`.
pub async fn health_handler() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: get_uptime_secs(),
    })
}

/// Endpoint handler for Kubernetes readiness probe `/ready`.
///
/// Returns HTTP 200 OK when active and healthy.
/// Returns HTTP 503 Service Unavailable when in Graceful Drain Mode.
pub async fn ready_handler() -> Response {
    if is_draining() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({
                "status": "draining",
                "message": "Node is in graceful drain mode, not accepting new connections"
            })),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(ReadyResponse {
            status: "ready",
            uptime_secs: get_uptime_secs(),
        }),
    )
        .into_response()
}
