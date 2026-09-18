use crate::state::ConnectionState;
use alvio_core::{PeerId, RoomId, TrackId};
use alvio_protocol::{PeerInfo, TrackInfo};
use serde::{Deserialize, Serialize};

/// Strongly-typed asynchronous events emitted by the client core.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum ClientEvent {
    /// Connected to signaling gateway and acknowledged.
    Connected { peer_id: PeerId, node_id: String },
    /// Successfully joined a room with initial state snapshot.
    RoomJoined {
        room_id: RoomId,
        self_peer_id: PeerId,
        peers: Vec<PeerInfo>,
        active_tracks: Vec<TrackInfo>,
    },
    /// A new remote peer joined the room.
    PeerJoined(PeerInfo),
    /// A remote peer left the room.
    PeerLeft { peer_id: PeerId, reason: String },
    /// A remote peer published a media track.
    TrackPublished(TrackInfo),
    /// A media track was unpublished.
    TrackUnpublished(TrackId),
    /// Received application data message from another peer.
    DataReceived {
        source_peer_id: PeerId,
        payload: String,
    },
    /// Server issued a remote SDP offer (e.g. for track subscription).
    RemoteOffer { sdp: String },
    /// Server answered a local SDP offer.
    RemoteAnswer { sdp: String },
    /// Server trickle ICE candidate.
    RemoteCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u32>,
    },
    /// Connection lifecycle state changed.
    StateChanged(ConnectionState),
    /// Server or signaling protocol error.
    Error { code: String, message: String },
    /// Client disconnected from gateway.
    Disconnected,
}
