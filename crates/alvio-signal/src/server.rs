use crate::handler::{ws_upgrade_handler, SignalState};
use crate::registry::RoomRegistry;
use axum::{
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use serde_json::json;

pub fn create_signaling_router(registry: RoomRegistry, node_id: String) -> Router {
    let state = SignalState {
        registry,
        node_id,
    };

    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/ws", get(ws_upgrade_handler))
        .with_state(state)
}

async fn root_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html>
<head><title>AlvioRelay Media Server</title></head>
<body style="font-family: system-ui, sans-serif; background: #0f172a; color: #f8fafc; display: flex; justify-content: center; align-items: center; height: 90vh;">
  <div style="text-align: center; border: 1px solid #334155; padding: 2rem 3rem; border-radius: 12px; background: #1e293b; box-shadow: 0 10px 25px rgba(0,0,0,0.5);">
    <h1 style="margin: 0; color: #38bdf8;">AlvioRelay</h1>
    <p style="color: #94a3b8; margin: 0.5rem 0 1.5rem 0;">Rust-Native Self-Hosted Real-Time Media Infrastructure</p>
    <div style="display: inline-block; padding: 0.25rem 0.75rem; background: #065f46; color: #34d399; border-radius: 999px; font-size: 0.875rem; font-weight: 600;">Status: Online & Ready</div>
    <p style="font-size: 0.85rem; color: #64748b; margin-top: 1.5rem;">Connect via WebSocket at <code>/ws</code> or broadcast via WHIP at <code>/whip/{room_id}</code></p>
  </div>
</body>
</html>"#)
}

async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }))
}

async fn ready_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ready"
    }))
}
