use alvio_core::{AlvioError, AlvioResult, PeerId, RoomId, TrackId};
use alvio_protocol::{PeerInfo, SignalEnvelope, SignalMessage, TrackInfo};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, info};

/// Represents an active connected client peer in a Room.
pub struct PeerSession {
    pub id: PeerId,
    pub name: String,
    pub metadata: Option<String>,
    pub tx: UnboundedSender<SignalEnvelope>,
}

impl PeerSession {
    pub fn new(
        id: PeerId,
        name: String,
        metadata: Option<String>,
        tx: UnboundedSender<SignalEnvelope>,
    ) -> Self {
        Self {
            id,
            name,
            metadata,
            tx,
        }
    }

    /// Sends a signaling message to this peer.
    pub fn send(&self, msg: SignalMessage) -> bool {
        let envelope = SignalEnvelope::new(msg);
        self.tx.send(envelope).is_ok()
    }

    pub fn info(&self) -> PeerInfo {
        PeerInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Represents an active media room holding in-memory state.
pub struct RoomSession {
    pub id: RoomId,
    pub peers: DashMap<PeerId, Arc<PeerSession>>,
    pub tracks: DashMap<TrackId, TrackInfo>,
    pub max_peers: usize,
    pub created_at: Instant,
}

impl RoomSession {
    pub fn new(id: RoomId, max_peers: usize) -> Self {
        Self {
            id,
            peers: DashMap::new(),
            tracks: DashMap::new(),
            max_peers,
            created_at: Instant::now(),
        }
    }

    /// Add a peer to this room.
    pub fn join_peer(&self, peer: Arc<PeerSession>) -> AlvioResult<()> {
        if self.peers.len() >= self.max_peers {
            return Err(AlvioError::RoomFull(self.id.clone()));
        }

        let info = peer.info();
        self.peers.insert(peer.id.clone(), peer);

        let sender_id = info.id.clone();
        self.broadcast(
            Some(&sender_id),
            SignalMessage::PeerJoined { peer: info },
        );

        Ok(())
    }

    /// Remove a peer from this room.
    pub fn leave_peer(&self, peer_id: &PeerId, reason: &str) -> Option<Arc<PeerSession>> {
        if let Some((_, peer)) = self.peers.remove(peer_id) {
            info!(room = %self.id, peer = %peer_id, reason = %reason, "Peer left room");

            let tracks_to_remove: Vec<TrackId> = self
                .tracks
                .iter()
                .filter(|entry| entry.value().peer_id == *peer_id)
                .map(|entry| entry.key().clone())
                .collect();

            for track_id in tracks_to_remove {
                self.unpublish_track(&track_id);
            }

            self.broadcast(
                None,
                SignalMessage::PeerLeft {
                    peer_id: peer_id.clone(),
                    reason: reason.to_string(),
                },
            );

            Some(peer)
        } else {
            None
        }
    }

    /// Register a new track publication in this room.
    pub fn publish_track(&self, track: TrackInfo) {
        self.tracks.insert(track.id.clone(), track.clone());
        let publisher_id = track.peer_id.clone();
        self.broadcast(
            Some(&publisher_id),
            SignalMessage::TrackPublished { track },
        );
    }

    /// Remove a published track from this room.
    pub fn unpublish_track(&self, track_id: &TrackId) -> Option<TrackInfo> {
        if let Some((_, track)) = self.tracks.remove(track_id) {
            self.broadcast(
                Some(&track.peer_id),
                SignalMessage::TrackUnpublished {
                    track_id: track_id.clone(),
                },
            );
            Some(track)
        } else {
            None
        }
    }

    /// Broadcasts a message to peers in this room, optionally excluding sender.
    pub fn broadcast(&self, exclude_peer_id: Option<&PeerId>, msg: SignalMessage) {
        let envelope = SignalEnvelope::new(msg);
        for entry in self.peers.iter() {
            if let Some(excluded) = exclude_peer_id {
                if entry.key() == excluded {
                    continue;
                }
            }
            let _ = entry.value().tx.send(envelope.clone());
        }
    }

    /// Sends a direct data message to targeted peers.
    pub fn send_direct_data(&self, source_peer_id: &PeerId, destination_peer_ids: &[PeerId], payload: &str) {
        let msg = SignalMessage::DataReceived {
            source_peer_id: source_peer_id.clone(),
            payload: payload.to_string(),
        };
        let envelope = SignalEnvelope::new(msg);

        if destination_peer_ids.is_empty() {
            for entry in self.peers.iter() {
                if entry.key() != source_peer_id {
                    let _ = entry.value().tx.send(envelope.clone());
                }
            }
        } else {
            for dest in destination_peer_ids {
                if let Some(peer) = self.peers.get(dest) {
                    let _ = peer.tx.send(envelope.clone());
                }
            }
        }
    }

    /// Current snapshot of peers and active tracks.
    pub fn get_snapshot(&self) -> (Vec<PeerInfo>, Vec<TrackInfo>) {
        let peers = self.peers.iter().map(|e| e.value().info()).collect();
        let tracks = self.tracks.iter().map(|e| e.value().clone()).collect();
        (peers, tracks)
    }

    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }
}

/// Global in-memory Room Registry (Zero Mandatory Database).
#[derive(Clone)]
pub struct RoomRegistry {
    rooms: Arc<DashMap<RoomId, Arc<RoomSession>>>,
    max_peers_per_room: usize,
}

impl RoomRegistry {
    pub fn new(max_peers_per_room: usize) -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
            max_peers_per_room,
        }
    }

    /// Gets existing room or creates a new one in-memory.
    pub fn get_or_create(&self, room_id: &RoomId) -> Arc<RoomSession> {
        self.rooms
            .entry(room_id.clone())
            .or_insert_with(|| {
                debug!(room = %room_id, "Creating new in-memory room session");
                Arc::new(RoomSession::new(room_id.clone(), self.max_peers_per_room))
            })
            .clone()
    }

    /// Finds an existing room if active.
    pub fn get(&self, room_id: &RoomId) -> Option<Arc<RoomSession>> {
        self.rooms.get(room_id).map(|r| r.clone())
    }

    /// Cleans up empty rooms to free memory.
    pub fn purge_empty_rooms(&self) -> usize {
        let mut purged = 0;
        self.rooms.retain(|id, room| {
            if room.is_empty() {
                debug!(room = %id, "Purging empty in-memory room");
                purged += 1;
                false
            } else {
                true
            }
        });
        purged
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
}
