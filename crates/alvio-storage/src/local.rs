use crate::backend::{StorageBackend, StorageMetadata};
use crate::error::{StorageError, StorageResult};
use async_trait::async_trait;
use bytes::Bytes;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;
use tracing::debug;

/// Local filesystem storage backend.
///
/// Writes recordings directly to a local volume or folder with zero external cloud dependencies.
pub struct LocalStorage {
    root_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        let clean_path = path.trim_start_matches(['/', '\\']);
        self.root_dir.join(clean_path)
    }

    fn detect_content_type(path: &str) -> &'static str {
        if path.ends_with(".mp4") {
            "video/mp4"
        } else if path.ends_with(".webm") {
            "video/webm"
        } else if path.ends_with(".m3u8") {
            "application/vnd.apple.mpegurl"
        } else if path.ends_with(".ts") {
            "video/mp2t"
        } else if path.ends_with(".json") {
            "application/json"
        } else {
            "application/octet-stream"
        }
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn put(&self, path: &str, data: Bytes) -> StorageResult<StorageMetadata> {
        let full_path = self.resolve_path(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let size_bytes = data.len() as u64;
        fs::write(&full_path, &data).await?;
        debug!(path = %full_path.display(), size_bytes, "Wrote local storage asset");

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(StorageMetadata {
            path: path.to_string(),
            size_bytes,
            content_type: Self::detect_content_type(path).to_string(),
            created_at: now,
        })
    }

    async fn get(&self, path: &str) -> StorageResult<Bytes> {
        let full_path = self.resolve_path(path);
        if !full_path.exists() {
            return Err(StorageError::NotFound(path.to_string()));
        }

        let data = fs::read(&full_path).await?;
        Ok(Bytes::from(data))
    }

    async fn delete(&self, path: &str) -> StorageResult<()> {
        let full_path = self.resolve_path(path);
        if full_path.exists() {
            fs::remove_file(&full_path).await?;
            debug!(path = %full_path.display(), "Deleted local storage asset");
        }
        Ok(())
    }

    async fn exists(&self, path: &str) -> StorageResult<bool> {
        let full_path = self.resolve_path(path);
        Ok(full_path.exists())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_local_storage_put_get_exists_delete() {
        let test_dir = std::env::temp_dir().join(format!(
            "alvio_storage_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let storage = LocalStorage::new(&test_dir);

        let test_path = "room-alpha/session-1/video.mp4";
        let test_data = Bytes::from_static(b"fake-mp4-video-stream-content");

        assert!(!storage.exists(test_path).await.unwrap());

        let meta = storage.put(test_path, test_data.clone()).await.unwrap();
        assert_eq!(meta.path, test_path);
        assert_eq!(meta.size_bytes, test_data.len() as u64);
        assert_eq!(meta.content_type, "video/mp4");

        assert!(storage.exists(test_path).await.unwrap());

        let fetched = storage.get(test_path).await.unwrap();
        assert_eq!(fetched, test_data);

        storage.delete(test_path).await.unwrap();
        assert!(!storage.exists(test_path).await.unwrap());
    }
}
