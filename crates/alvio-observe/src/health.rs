use crate::metrics::get_metrics;
use axum::{
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Instant;

static SERVER_START_TIME: OnceLock<Instant> = OnceLock::new();

pub fn init_server_start_time() {
    SERVER_START_TIME.get_or_init(Instant::now);
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
pub async fn ready_handler() -> impl IntoResponse {
    Json(ReadyResponse {
        status: "ready",
        uptime_secs: get_uptime_secs(),
    })
}
