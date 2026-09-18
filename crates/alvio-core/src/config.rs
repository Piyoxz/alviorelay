use crate::error::{AlvioError, AlvioResult};
use crate::types::IceServerConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlvioConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub rtc: RtcConfig,
    #[serde(default)]
    pub room: RoomConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub egress: EgressConfig,
    #[serde(default)]
    pub webhooks: WebhooksConfig,
}

impl Default for AlvioConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            rtc: RtcConfig::default(),
            room: RoomConfig::default(),
            auth: AuthConfig::default(),
            egress: EgressConfig::default(),
            webhooks: WebhooksConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_ws_port")]
    pub ws_port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            node_id: default_node_id(),
            bind_address: default_bind_address(),
            http_port: default_http_port(),
            ws_port: default_ws_port(),
            log_level: default_log_level(),
        }
    }
}

fn default_node_id() -> String {
    "alvio-node-01".to_string()
}
fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}
fn default_http_port() -> u16 {
    7880
}
fn default_ws_port() -> u16 {
    7880
}
fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtcConfig {
    #[serde(default = "default_udp_port")]
    pub udp_port: u16,
    #[serde(default = "default_tcp_port")]
    pub tcp_port: u16,
    #[serde(default)]
    pub use_external_ip: bool,
    pub external_ip: Option<String>,
    #[serde(default = "default_ice_servers")]
    pub ice_servers: Vec<IceServerConfig>,
}

impl Default for RtcConfig {
    fn default() -> Self {
        Self {
            udp_port: default_udp_port(),
            tcp_port: default_tcp_port(),
            use_external_ip: false,
            external_ip: None,
            ice_servers: default_ice_servers(),
        }
    }
}

fn default_udp_port() -> u16 {
    7882
}
fn default_tcp_port() -> u16 {
    7881
}
fn default_ice_servers() -> Vec<IceServerConfig> {
    vec![IceServerConfig {
        urls: vec!["stun:stun.l.google.com:19302".to_string()],
        username: None,
        credential: None,
    }]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomConfig {
    #[serde(default = "default_empty_timeout")]
    pub empty_timeout_secs: u64,
    #[serde(default = "default_max_peers")]
    pub max_peers_per_room: usize,
    #[serde(default = "default_true")]
    pub enable_data_channels: bool,
}

impl Default for RoomConfig {
    fn default() -> Self {
        Self {
            empty_timeout_secs: default_empty_timeout(),
            max_peers_per_room: default_max_peers(),
            enable_data_channels: default_true(),
        }
    }
}

fn default_empty_timeout() -> u64 {
    300
}
fn default_max_peers() -> usize {
    100
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_auth_provider")]
    pub provider: String,
    pub secret_key: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            provider: default_auth_provider(),
            secret_key: None,
        }
    }
}

fn default_auth_provider() -> String {
    "no_auth".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_storage_backend")]
    pub storage_backend: String,
    #[serde(default = "default_storage_dir")]
    pub local_storage_dir: String,
}

impl Default for EgressConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            storage_backend: default_storage_backend(),
            local_storage_dir: default_storage_dir(),
        }
    }
}

fn default_storage_backend() -> String {
    "local".to_string()
}
fn default_storage_dir() -> String {
    "./recordings".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhooksConfig {
    #[serde(default)]
    pub enabled: bool,
    pub endpoint_url: Option<String>,
    pub secret: Option<String>,
}

impl Default for WebhooksConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            endpoint_url: None,
            secret: None,
        }
    }
}

impl AlvioConfig {
    /// Loads configuration from a file path or falls back to default.
    /// Environment variables prefixed with `ALVIO_` override values.
    pub fn load_from_file_or_default<P: AsRef<Path>>(path: P) -> AlvioResult<Self> {
        let mut config = if path.as_ref().exists() {
            let content = fs::read_to_string(path)
                .map_err(|e| AlvioError::Config(format!("Failed to read config file: {e}")))?;
            toml::from_str(&content)
                .map_err(|e| AlvioError::Config(format!("Failed to parse TOML config: {e}")))?
        } else {
            Self::default()
        };

        config.apply_env_overrides();
        config.validate()?;
        Ok(config)
    }

    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = env::var("ALVIO_NODE_ID") {
            self.server.node_id = val;
        }
        if let Ok(val) = env::var("ALVIO_BIND_ADDRESS") {
            self.server.bind_address = val;
        }
        if let Ok(val) = env::var("ALVIO_HTTP_PORT") {
            if let Ok(port) = val.parse() {
                self.server.http_port = port;
                self.server.ws_port = port;
            }
        }
        if let Ok(val) = env::var("ALVIO_LOG_LEVEL") {
            self.server.log_level = val;
        }
        if let Ok(val) = env::var("ALVIO_RTC_UDP_PORT") {
            if let Ok(port) = val.parse() {
                self.rtc.udp_port = port;
            }
        }
        if let Ok(val) = env::var("ALVIO_RTC_EXTERNAL_IP") {
            if !val.trim().is_empty() {
                self.rtc.external_ip = Some(val);
                self.rtc.use_external_ip = true;
            }
        }
        if let Ok(val) = env::var("ALVIO_AUTH_PROVIDER") {
            self.auth.provider = val;
        }
        if let Ok(val) = env::var("ALVIO_AUTH_SECRET_KEY") {
            self.auth.secret_key = Some(val);
        }
        if let Ok(val) = env::var("ALVIO_STORAGE_BACKEND") {
            self.egress.storage_backend = val;
        }
        if let Ok(val) = env::var("ALVIO_STORAGE_PATH") {
            self.egress.local_storage_dir = val;
        }
    }

    pub fn validate(&self) -> AlvioResult<()> {
        if self.server.http_port == 0 {
            return Err(AlvioError::Config("HTTP port cannot be 0".to_string()));
        }
        if self.rtc.udp_port == 0 {
            return Err(AlvioError::Config("RTC UDP port cannot be 0".to_string()));
        }
        if self.auth.provider == "jwt" && self.auth.secret_key.is_none() {
            return Err(AlvioError::Config(
                "JWT auth provider selected but secret_key is not configured".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_valid() {
        let config = AlvioConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.server.http_port, 7880);
        assert_eq!(config.rtc.udp_port, 7882);
    }

    #[test]
    fn test_toml_parsing() {
        let toml_str = r#"
            [server]
            node_id = "test-node"
            http_port = 8080

            [rtc]
            udp_port = 9000
        "#;
        let config: AlvioConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.server.node_id, "test-node");
        assert_eq!(config.server.http_port, 8080);
        assert_eq!(config.rtc.udp_port, 9000);
    }
}
