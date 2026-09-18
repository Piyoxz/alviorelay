use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Storage backend error: {0}")]
    Backend(String),
}

pub type StorageResult<T> = Result<T, StorageError>;
