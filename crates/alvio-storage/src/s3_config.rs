/// Configuration for AWS S3 / MinIO / Cloudflare R2 compatible storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S3StorageConfig {
    pub bucket: String,
    pub region: String,
    pub endpoint: Option<String>,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub force_path_style: bool,
}

impl S3StorageConfig {
    pub fn new(
        bucket: impl Into<String>,
        region: impl Into<String>,
        access_key_id: impl Into<String>,
        secret_access_key: impl Into<String>,
    ) -> Self {
        Self {
            bucket: bucket.into(),
            region: region.into(),
            endpoint: None,
            access_key_id: access_key_id.into(),
            secret_access_key: secret_access_key.into(),
            force_path_style: false,
        }
    }

    pub fn with_custom_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self.force_path_style = true; // Typically required for MinIO
        self
    }
}
