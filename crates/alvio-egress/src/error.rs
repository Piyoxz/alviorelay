use thiserror::Error;

#[derive(Error, Debug)]
pub enum EgressError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Storage error: {0}")]
    Storage(#[from] alvio_storage::StorageError),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session already active for room: {0}")]
    SessionAlreadyActive(String),

    #[error("FFmpeg process error: {0}")]
    Ffmpeg(String),

    #[error("Egress channel closed")]
    ChannelClosed,
}

pub type EgressResult<T> = Result<T, EgressError>;
