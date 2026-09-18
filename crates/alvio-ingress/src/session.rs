use alvio_core::{PeerId, RoomId};
use dashmap::DashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Metadata and state for an active WHIP publishing session.
#[derive(Debug, Clone)]
pub struct WhipSession {
    pub resource_id: String,
    pub peer_id: PeerId,
    pub room_id: RoomId,
    pub stream_key: Option<String>,
    pub created_at: u64,
    pub etag: String,
    pub candidates: Arc<Mutex<Vec<String>>>,
}

impl WhipSession {
    pub fn new(room_id: RoomId, peer_id: PeerId, stream_key: Option<String>) -> Self {
        let resource_id = format!("whip_res_{}", Uuid::new_v4().simple());
        let etag = format!("\"whip_{}\"", Uuid::new_v4().simple());
        Self {
            resource_id,
            peer_id,
            room_id,
            stream_key,
            created_at: current_timestamp(),
            etag,
            candidates: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_candidate(&self, candidate_sdp: String) {
        self.candidates.lock().push(candidate_sdp);
    }

    pub fn candidate_count(&self) -> usize {
        self.candidates.lock().len()
    }
}

/// Thread-safe registry managing active WHIP broadcast sessions.
#[derive(Clone, Default)]
pub struct WhipRegistry {
    sessions: Arc<DashMap<String, Arc<WhipSession>>>,
    peer_to_resource: Arc<DashMap<PeerId, String>>,
}

impl WhipRegistry {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(DashMap::new()),
            peer_to_resource: Arc::new(DashMap::new()),
        }
    }

    /// Registers a new active WHIP publishing session.
    pub fn create_session(
        &self,
        room_id: RoomId,
        peer_id: PeerId,
        stream_key: Option<String>,
    ) -> Arc<WhipSession> {
        let session = Arc::new(WhipSession::new(room_id, peer_id.clone(), stream_key));
        self.sessions
            .insert(session.resource_id.clone(), Arc::clone(&session));
        self.peer_to_resource
            .insert(peer_id, session.resource_id.clone());
        session
    }

    /// Finds a WHIP session by its resource ID.
    pub fn get(&self, resource_id: &str) -> Option<Arc<WhipSession>> {
        self.sessions.get(resource_id).map(|r| Arc::clone(&r))
    }

    /// Removes and tears down a WHIP session by its resource ID.
    pub fn remove(&self, resource_id: &str) -> Option<Arc<WhipSession>> {
        if let Some((_, session)) = self.sessions.remove(resource_id) {
            self.peer_to_resource.remove(&session.peer_id);
            Some(session)
        } else {
            None
        }
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whip_session_and_registry_lifecycle() {
        let registry = WhipRegistry::new();
        assert_eq!(registry.active_count(), 0);

        let room_id = RoomId::from("livestream-studio");
        let peer_id = PeerId::from("obs-encoder-1");
        let session = registry.create_session(room_id.clone(), peer_id.clone(), Some("secret_key_123".into()));

        assert_eq!(registry.active_count(), 1);
        assert_eq!(session.room_id, room_id);
        assert_eq!(session.peer_id, peer_id);
        assert_eq!(session.stream_key.as_deref(), Some("secret_key_123"));

        // Add trickle ICE candidate
        session.add_candidate("candidate:1 1 UDP 2130706431 192.168.1.50 50000 typ host".to_string());
        assert_eq!(session.candidate_count(), 1);

        // Fetch session
        let found = registry.get(&session.resource_id).unwrap();
        assert_eq!(found.resource_id, session.resource_id);

        // Remove session
        let removed = registry.remove(&session.resource_id).unwrap();
        assert_eq!(removed.resource_id, session.resource_id);
        assert_eq!(registry.active_count(), 0);
        assert!(registry.get(&session.resource_id).is_none());
    }
}
