use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for an AlvioRelay Room.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomId(String);

impl RoomId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for RoomId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for RoomId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Unique identifier for an active Peer in a Room.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(String);

impl PeerId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PeerId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for PeerId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Unique identifier for a media track.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(String);

impl TrackId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TrackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TrackId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for TrackId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Media stream kind published by a Peer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamKind {
    Audio,
    Video,
    Screen,
    ScreenAudio,
}

impl fmt::Display for StreamKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audio => write!(f, "audio"),
            Self::Video => write!(f, "video"),
            Self::Screen => write!(f, "screen"),
            Self::ScreenAudio => write!(f, "screen_audio"),
        }
    }
}

/// Quality layer for simulcast streams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamLayer {
    Low,
    Medium,
    High,
}

impl fmt::Display for StreamLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
        }
    }
}

/// WebRTC Peer Connection State.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Connecting,
    Connected,
    Reconnecting,
    Disconnected,
    Failed,
}

/// Configuration for an ICE/STUN/TURN server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IceServerConfig {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_types() {
        let room = RoomId::from("room-alpha");
        assert_eq!(room.as_str(), "room-alpha");
        assert_eq!(format!("{room}"), "room-alpha");

        let peer = PeerId::from("peer-42");
        assert_eq!(peer.as_str(), "peer-42");

        let track = TrackId::from("track-camera-01");
        assert_eq!(track.as_str(), "track-camera-01");
    }

    #[test]
    fn test_stream_kind_and_layers() {
        assert_eq!(StreamKind::Video.to_string(), "video");
        assert_eq!(StreamLayer::High.to_string(), "high");
        assert!(StreamLayer::High > StreamLayer::Low);
    }
}
