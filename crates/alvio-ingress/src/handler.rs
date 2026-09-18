use crate::error::IngressError;
use crate::session::WhipRegistry;
use alvio_core::{PeerId, RoomId};
use alvio_signal::RoomRegistry;
use alvio_webrtc::AlvioTransport;
use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Shared application state for the WHIP ingress service.
#[derive(Clone)]
pub struct WhipState {
    pub whip_registry: WhipRegistry,
    pub room_registry: RoomRegistry,
    pub rtc_bind_addr: SocketAddr,
    pub expected_bearer_token: Option<String>,
}

impl WhipState {
    pub fn new(
        whip_registry: WhipRegistry,
        room_registry: RoomRegistry,
        rtc_bind_addr: SocketAddr,
        expected_bearer_token: Option<String>,
    ) -> Self {
        Self {
            whip_registry,
            room_registry,
            rtc_bind_addr,
            expected_bearer_token,
        }
    }
}

/// Helper function to attach standard CORS headers to WHIP responses.
fn add_cors_headers(headers: &mut HeaderMap) {
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("POST, DELETE, PATCH, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Content-Type, Authorization, If-Match"),
    );
    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static("Location, Link, ETag"),
    );
}

/// Standard WHIP OPTIONS preflight handler for CORS and protocol discovery.
pub async fn whip_options_handler() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    add_cors_headers(&mut headers);
    headers.insert(header::ALLOW, HeaderValue::from_static("POST, DELETE, PATCH, OPTIONS"));
    headers.insert(header::HeaderName::from_static("accept-post"), HeaderValue::from_static("application/sdp"));
    headers.insert(
        header::HeaderName::from_static("accept-patch"),
        HeaderValue::from_static("application/trickle-ice-sdpfrag"),
    );
    (StatusCode::NO_CONTENT, headers)
}

/// Standard WHIP POST handler accepting an SDP Offer and returning an SDP Answer with 201 Created.
pub async fn whip_publish_handler(
    State(state): State<WhipState>,
    Path(room_id_str): Path<String>,
    headers: HeaderMap,
    sdp_offer: String,
) -> Result<Response, IngressError> {
    // 1. Verify Content-Type header
    if let Some(content_type) = headers.get(header::CONTENT_TYPE) {
        let ct = content_type.to_str().unwrap_or_default();
        if !ct.starts_with("application/sdp") && !ct.starts_with("text/plain") {
            return Err(IngressError::UnsupportedContentType(ct.to_string()));
        }
    } else {
        return Err(IngressError::UnsupportedContentType("missing Content-Type".to_string()));
    }

    // 2. Validate Authentication (if enabled)
    let stream_key = if let Some(ref expected_token) = state.expected_bearer_token {
        let auth_hdr = headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .unwrap_or_default();

        let token = auth_hdr.strip_prefix("Bearer ").map(|t| t.trim());
        if token != Some(expected_token.as_str()) {
            warn!(room = %room_id_str, "WHIP ingress rejected: unauthorized token");
            return Err(IngressError::Unauthorized);
        }
        token.map(ToString::to_string)
    } else {
        headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.trim().to_string())
    };

    // 3. Validate SDP Offer
    let sdp_trimmed = sdp_offer.trim();
    if sdp_trimmed.is_empty() {
        return Err(IngressError::InvalidSdp("SDP offer cannot be empty".to_string()));
    }

    let room_id = RoomId::from(room_id_str.clone());
    let peer_id = PeerId::from(format!("whip_{}", Uuid::new_v4().simple()));

    // 4. Verify room capacity and register publisher peer in signaling registry
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let room = state.room_registry.get_or_create(&room_id);
    let peer = Arc::new(alvio_signal::PeerSession::new(
        peer_id.clone(),
        "WHIP Broadcaster".to_string(),
        None,
        tx,
    ));
    room.join_peer(peer)
        .map_err(|e| IngressError::RoomCapacityExceeded(e.to_string()))?;

    // 5. Initialize WebRTC Transport and process remote SDP offer
    let mut transport = AlvioTransport::new(state.rtc_bind_addr)
        .map_err(|e| IngressError::Transport(e.to_string()))?;

    let sdp_answer = match transport.accept_remote_offer(sdp_trimmed) {
        Ok(ans) => ans,
        Err(e) => {
            // Rollback peer registration on SDP failure
            if let Some(r) = state.room_registry.get(&room_id) {
                r.leave_peer(&peer_id, "SDP negotiation failure");
            }
            return Err(IngressError::InvalidSdp(e.to_string()));
        }
    };

    // 6. Register WHIP Session
    let session = state
        .whip_registry
        .create_session(room_id.clone(), peer_id.clone(), stream_key);

    info!(
        resource_id = %session.resource_id,
        %room_id,
        %peer_id,
        "WHIP session successfully established via 201 Created"
    );

    // 7. Format WHIP 201 Created HTTP Response
    let mut resp_headers = HeaderMap::new();
    add_cors_headers(&mut resp_headers);
    resp_headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/sdp"));

    let resource_url = format!("/whip/resource/{}", session.resource_id);
    if let Ok(loc_val) = HeaderValue::from_str(&resource_url) {
        resp_headers.insert(header::LOCATION, loc_val);
    }

    let link_hdr = format!("<{}>; rel=\"urn:ietf:params:whip:resource-url\"", resource_url);
    if let Ok(link_val) = HeaderValue::from_str(&link_hdr) {
        resp_headers.insert(header::LINK, link_val);
    }

    if let Ok(etag_val) = HeaderValue::from_str(&session.etag) {
        resp_headers.insert(header::ETAG, etag_val);
    }

    Ok((StatusCode::CREATED, resp_headers, sdp_answer).into_response())
}

/// Standard WHIP DELETE handler to terminate an active publisher session.
pub async fn whip_delete_handler(
    State(state): State<WhipState>,
    Path(resource_id): Path<String>,
) -> Result<Response, IngressError> {
    let session = state
        .whip_registry
        .remove(&resource_id)
        .ok_or_else(|| IngressError::SessionNotFound(resource_id.clone()))?;

    // Disconnect peer from signaling room registry
    if let Some(room) = state.room_registry.get(&session.room_id) {
        room.leave_peer(&session.peer_id, "WHIP publishing session terminated via DELETE");
    }

    info!(
        %resource_id,
        room_id = %session.room_id,
        peer_id = %session.peer_id,
        "WHIP publishing session terminated via DELETE"
    );

    let mut headers = HeaderMap::new();
    add_cors_headers(&mut headers);
    Ok((StatusCode::NO_CONTENT, headers).into_response())
}

/// Standard WHIP PATCH handler to receive remote trickle ICE candidate fragments.
pub async fn whip_patch_handler(
    State(state): State<WhipState>,
    Path(resource_id): Path<String>,
    headers: HeaderMap,
    candidate_body: String,
) -> Result<Response, IngressError> {
    // 1. Verify Content-Type
    if let Some(ct) = headers.get(header::CONTENT_TYPE) {
        let ct_str = ct.to_str().unwrap_or_default();
        if !ct_str.starts_with("application/trickle-ice-sdpfrag") && !ct_str.starts_with("text/plain") {
            return Err(IngressError::UnsupportedContentType(ct_str.to_string()));
        }
    }

    // 2. Fetch active session
    let session = state
        .whip_registry
        .get(&resource_id)
        .ok_or_else(|| IngressError::SessionNotFound(resource_id.clone()))?;

    let fragment_trimmed = candidate_body.trim();
    if !fragment_trimmed.is_empty() {
        debug!(%resource_id, len = fragment_trimmed.len(), "Received trickle ICE candidate fragment via PATCH");
        session.add_candidate(fragment_trimmed.to_string());
    }

    let mut resp_headers = HeaderMap::new();
    add_cors_headers(&mut resp_headers);
    Ok((StatusCode::NO_CONTENT, resp_headers).into_response())
}
