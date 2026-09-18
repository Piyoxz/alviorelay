use crate::error::StorageResult;
use async_trait::async_trait;
use bytes::Bytes;

/// Metadata describing a stored media asset or recording.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageMetadata {
    pub path: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub created_at: u64,
}

/// Pluggable asynchronous storage backend for recordings and session archives.
#[async_trait]
pub trait StorageBackend: Send + Sync + 'static {
    /// Stores binary data at the given destination path.
    async fn put(&self, path: &str, data: Bytes) -> StorageResult<StorageMetadata>;

    /// Reads binary data from the given path.
    async fn get(&self, path: &str) -> StorageResult<Bytes>;

    /// Removes an object from storage.
    async fn delete(&self, path: &str) -> StorageResult<()>;

    /// Checks whether an object exists.
    async fn exists(&self, path: &str) -> StorageResult<bool>;
}
