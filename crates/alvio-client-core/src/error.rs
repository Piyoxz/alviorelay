use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Client is not connected to signaling gateway")]
    NotConnected,

    #[error("Client is already connected")]
    AlreadyConnected,

    #[error("Client is already in a room")]
    AlreadyInRoom,

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Signaling error: {0}")]
    SignalingError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Track not found: {0}")]
    TrackNotFound(String),

    #[error("Operation timed out")]
    Timeout,

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Channel closed")]
    ChannelClosed,
}
