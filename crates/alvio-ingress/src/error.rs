use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type IngressResult<T> = Result<T, IngressError>;

#[derive(Debug, Error)]
pub enum IngressError {
    #[error("Invalid SDP offer: {0}")]
    InvalidSdp(String),

    #[error("Unsupported Content-Type: {0}. Expected 'application/sdp' or 'application/trickle-ice-sdpfrag'")]
    UnsupportedContentType(String),

    #[error("WHIP resource session not found: {0}")]
    SessionNotFound(String),

    #[error("Unauthorized: invalid or missing stream key / bearer token")]
    Unauthorized,

    #[error("Room capacity exceeded for room: {0}")]
    RoomCapacityExceeded(String),

    #[error("WebRTC transport error: {0}")]
    Transport(String),

    #[error("Internal ingress error: {0}")]
    Internal(String),
}

impl IntoResponse for IngressError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::InvalidSdp(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::UnsupportedContentType(msg) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, msg.clone()),
            Self::SessionNotFound(id) => {
                (StatusCode::NOT_FOUND, format!("Resource '{id}' not found"))
            }
            Self::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Authentication required".to_string(),
            ),
            Self::RoomCapacityExceeded(room) => (
                StatusCode::CONFLICT,
                format!("Room '{room}' capacity exceeded"),
            ),
            Self::Transport(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        let body = Json(json!({
            "error": self.to_string(),
            "message": message,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ingress_error_status_codes() {
        let err1 = IngressError::InvalidSdp("bad sdp".into());
        let resp1 = err1.into_response();
        assert_eq!(resp1.status(), StatusCode::BAD_REQUEST);

        let err2 = IngressError::UnsupportedContentType("application/json".into());
        let resp2 = err2.into_response();
        assert_eq!(resp2.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let err3 = IngressError::SessionNotFound("res_123".into());
        let resp3 = err3.into_response();
        assert_eq!(resp3.status(), StatusCode::NOT_FOUND);

        let err4 = IngressError::Unauthorized;
        let resp4 = err4.into_response();
        assert_eq!(resp4.status(), StatusCode::UNAUTHORIZED);

        let err5 = IngressError::RoomCapacityExceeded("room_1".into());
        let resp5 = err5.into_response();
        assert_eq!(resp5.status(), StatusCode::CONFLICT);
    }
}
