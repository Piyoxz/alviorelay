use crate::types::{PeerId, RoomId, TrackId};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type AlvioResult<T> = Result<T, AlvioError>;

#[derive(Error, Debug)]
pub enum AlvioError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Room '{0}' not found")]
    RoomNotFound(RoomId),

    #[error("Room '{0}' has reached maximum peer capacity")]
    RoomFull(RoomId),

    #[error("Peer '{0}' not found")]
    PeerNotFound(PeerId),

    #[error("Track '{0}' not found")]
    TrackNotFound(TrackId),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Authorization forbidden: {0}")]
    Authorization(String),

    #[error("Signaling protocol violation: {0}")]
    Signaling(String),

    #[error("WebRTC transport error: {0}")]
    Transport(String),

    #[error("Media routing error: {0}")]
    Media(String),

    #[error("Storage backend error: {0}")]
    Storage(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl AlvioError {
    /// Returns a stable, language-agnostic machine readable error code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Config(_) => "CONFIG_ERROR",
            Self::RoomNotFound(_) => "ROOM_NOT_FOUND",
            Self::RoomFull(_) => "ROOM_FULL",
            Self::PeerNotFound(_) => "PEER_NOT_FOUND",
            Self::TrackNotFound(_) => "TRACK_NOT_FOUND",
            Self::Authentication(_) => "AUTHENTICATION_FAILED",
            Self::Authorization(_) => "AUTHORIZATION_FORBIDDEN",
            Self::Signaling(_) => "SIGNALING_ERROR",
            Self::Transport(_) => "TRANSPORT_ERROR",
            Self::Media(_) => "MEDIA_ROUTING_ERROR",
            Self::Storage(_) => "STORAGE_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

/// Standard structured API error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl From<&AlvioError> for ErrorResponse {
    fn from(err: &AlvioError) -> Self {
        Self {
            error: ErrorDetail {
                code: err.code().to_string(),
                message: err.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_mapping() {
        let err = AlvioError::RoomNotFound(RoomId::from("dev-room"));
        assert_eq!(err.code(), "ROOM_NOT_FOUND");
        assert_eq!(err.to_string(), "Room 'dev-room' not found");

        let resp = ErrorResponse::from(&err);
        assert_eq!(resp.error.code, "ROOM_NOT_FOUND");
    }
}
