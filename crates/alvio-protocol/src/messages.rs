use alvio_core::{PeerId, RoomId, StreamKind, StreamLayer, TrackId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CURRENT_PROTOCOL_VERSION: u8 = 1;

/// Standard versioned envelope for all signaling communication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalEnvelope {
    pub version: u8,
    pub id: String,
    #[serde(flatten)]
    pub body: SignalMessage,
}

impl SignalEnvelope {
    pub fn new(body: SignalMessage) -> Self {
        Self {
            version: CURRENT_PROTOCOL_VERSION,
            id: format!("msg_{}", Uuid::new_v4().simple()),
            body,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Metadata information about a connected Peer in a Room.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: PeerId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
}

/// Metadata information about an active media track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: TrackId,
    pub peer_id: PeerId,
    pub kind: StreamKind,
    pub source: String,
    pub layers: Vec<StreamLayer>,
}

/// Typed signaling messages exchanged between Client and Server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum SignalMessage {
    // -------------------------------------------------------------
    // Client -> Server
    // -------------------------------------------------------------
    /// Initial client handshake with optional auth token.
    Connect {
        #[serde(skip_serializing_if = "Option::is_none")]
        token: Option<String>,
        client_version: String,
    },
    /// Request to join a specific room.
    Join {
        room_id: RoomId,
        peer_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<String>,
    },
    /// WebRTC SDP Offer from client.
    Offer {
        sdp: String,
    },
    /// WebRTC SDP Answer from client.
    Answer {
        sdp: String,
    },
    /// Trickle ICE candidate from client.
    Candidate {
        candidate: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sdp_mid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sdp_mline_index: Option<u32>,
    },
    /// Publish a local media track (camera, mic, screenshare).
    PublishTrack {
        kind: StreamKind,
        source: String,
        layers: Vec<StreamLayer>,
    },
    /// Unpublish a previously published media track.
    UnpublishTrack {
        track_id: TrackId,
    },
    /// Subscribe to a remote peer's media track.
    Subscribe {
        track_id: TrackId,
        #[serde(skip_serializing_if = "Option::is_none")]
        preferred_layer: Option<StreamLayer>,
    },
    /// Select or change target layer for a subscribed simulcast track.
    LayerSelect {
        track_id: TrackId,
        layer: StreamLayer,
    },
    /// Send application data message to other peers.
    DataMessage {
        destination_peer_ids: Vec<PeerId>,
        payload: String,
        reliable: bool,
    },
    /// Voluntarily leave the room.
    Leave,
    /// Heartbeat keepalive ping.
    Ping,

    // -------------------------------------------------------------
    // Server -> Client
    // -------------------------------------------------------------
    /// Handshake acknowledgment with assigned PeerId.
    Ack {
        peer_id: PeerId,
        node_id: String,
    },
    /// Notification of successful room join with snapshot of existing state.
    RoomJoined {
        room_id: RoomId,
        self_peer_id: PeerId,
        peers: Vec<PeerInfo>,
        active_tracks: Vec<TrackInfo>,
    },
    /// Broadcast that a new peer has joined the room.
    PeerJoined {
        peer: PeerInfo,
    },
    /// Broadcast that a peer has left the room.
    PeerLeft {
        peer_id: PeerId,
        reason: String,
    },
    /// Server-generated SDP Offer (for subscriptions / renegotiation).
    RemoteOffer {
        sdp: String,
    },
    /// Server response SDP Answer to a client offer.
    RemoteAnswer {
        sdp: String,
    },
    /// Server-generated ICE Candidate.
    RemoteCandidate {
        candidate: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        sdp_mid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        sdp_mline_index: Option<u32>,
    },
    /// Broadcast that a peer published a new track.
    TrackPublished {
        track: TrackInfo,
    },
    /// Broadcast that a track was removed.
    TrackUnpublished {
        track_id: TrackId,
    },
    /// Incoming data message routed from another peer.
    DataReceived {
        source_peer_id: PeerId,
        payload: String,
    },
    /// Error notification sent to the client.
    Error {
        code: String,
        message: String,
    },
    /// Heartbeat keepalive pong.
    Pong,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_serialization_roundtrip() {
        let msg = SignalMessage::Join {
            room_id: RoomId::from("conf-room-1"),
            peer_name: "Alice".to_string(),
            metadata: Some("{\"role\":\"host\"}".to_string()),
        };

        let env = SignalEnvelope::new(msg.clone());
        assert_eq!(env.version, CURRENT_PROTOCOL_VERSION);
        assert!(env.id.starts_with("msg_"));

        let json = env.to_json().expect("Serialization should succeed");
        assert!(json.contains("\"type\":\"join\""));
        assert!(json.contains("\"room_id\":\"conf-room-1\""));
        assert!(json.contains("\"peer_name\":\"Alice\""));

        let deserialized: SignalEnvelope = SignalEnvelope::from_json(&json).expect("Deserialization should succeed");
        assert_eq!(deserialized.version, env.version);
        assert_eq!(deserialized.body, msg);
    }

    #[test]
    fn test_track_published_envelope() {
        let track = TrackInfo {
            id: TrackId::from("track-video-01"),
            peer_id: PeerId::from("peer-bob"),
            kind: StreamKind::Video,
            source: "camera".to_string(),
            layers: vec![StreamLayer::Low, StreamLayer::High],
        };

        let env = SignalEnvelope::new(SignalMessage::TrackPublished { track: track.clone() });
        let json = env.to_json().unwrap();
        assert!(json.contains("\"type\":\"track_published\""));
        assert!(json.contains("\"track-video-01\""));

        let parsed = SignalEnvelope::from_json(&json).unwrap();
        if let SignalMessage::TrackPublished { track: parsed_track } = parsed.body {
            assert_eq!(parsed_track, track);
        } else {
            panic!("Expected TrackPublished variant");
        }
    }
}
