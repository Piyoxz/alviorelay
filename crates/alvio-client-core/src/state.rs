use serde::{Deserialize, Serialize};

/// Represents the connection lifecycle state of the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum ConnectionState {
    /// Client is disconnected and idle.
    #[default]
    Disconnected,
    /// Client is establishing signaling/peer connection.
    Connecting,
    /// Client is authenticated and connected to the signaling gateway.
    Connected,
    /// Client is actively participating in a room.
    InRoom,
    /// Connection was lost and the client is reconnecting with backoff.
    Reconnecting,
    /// Terminal failure state.
    Failed,
}
