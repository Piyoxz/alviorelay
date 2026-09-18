use alvio_core::{PeerId, RoomId, StreamKind, TrackId};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Standardized event types emitted across the AlvioRelay lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    RoomCreated,
    RoomDestroyed,
    PeerJoined,
    PeerLeft,
    TrackPublished,
    TrackUnpublished,
    RecordingStarted,
    RecordingCompleted,
    EgressFailed,
}

impl WebhookEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RoomCreated => "room.created",
            Self::RoomDestroyed => "room.destroyed",
            Self::PeerJoined => "peer.joined",
            Self::PeerLeft => "peer.left",
            Self::TrackPublished => "track.published",
            Self::TrackUnpublished => "track.unpublished",
            Self::RecordingStarted => "recording.started",
            Self::RecordingCompleted => "recording.completed",
            Self::EgressFailed => "egress.failed",
        }
    }
}

/// Standardized JSON webhook notification envelope.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    pub event: WebhookEventType,
    pub timestamp: u64,
    pub payload: Value,
}

impl WebhookEvent {
    pub fn new(event: WebhookEventType, payload: Value) -> Self {
        Self {
            id: format!("evt_{}", Uuid::new_v4().simple()),
            event,
            timestamp: current_timestamp(),
            payload,
        }
    }

    pub fn room_created(room_id: &RoomId) -> Self {
        Self::new(
            WebhookEventType::RoomCreated,
            json!({
                "room_id": room_id.as_str(),
            }),
        )
    }

    pub fn room_destroyed(room_id: &RoomId, duration_secs: u64) -> Self {
        Self::new(
            WebhookEventType::RoomDestroyed,
            json!({
                "room_id": room_id.as_str(),
                "duration_secs": duration_secs,
            }),
        )
    }

    pub fn peer_joined(
        room_id: &RoomId,
        peer_id: &PeerId,
        name: &str,
        metadata: Option<&str>,
    ) -> Self {
        Self::new(
            WebhookEventType::PeerJoined,
            json!({
                "room_id": room_id.as_str(),
                "peer_id": peer_id.as_str(),
                "name": name,
                "metadata": metadata,
            }),
        )
    }

    pub fn peer_left(
        room_id: &RoomId,
        peer_id: &PeerId,
        duration_secs: u64,
        reason: &str,
    ) -> Self {
        Self::new(
            WebhookEventType::PeerLeft,
            json!({
                "room_id": room_id.as_str(),
                "peer_id": peer_id.as_str(),
                "duration_secs": duration_secs,
                "reason": reason,
            }),
        )
    }

    pub fn track_published(
        room_id: &RoomId,
        peer_id: &PeerId,
        track_id: &TrackId,
        kind: StreamKind,
    ) -> Self {
        Self::new(
            WebhookEventType::TrackPublished,
            json!({
                "room_id": room_id.as_str(),
                "peer_id": peer_id.as_str(),
                "track_id": track_id.as_str(),
                "kind": kind,
            }),
        )
    }

    pub fn track_unpublished(
        room_id: &RoomId,
        peer_id: &PeerId,
        track_id: &TrackId,
    ) -> Self {
        Self::new(
            WebhookEventType::TrackUnpublished,
            json!({
                "room_id": room_id.as_str(),
                "peer_id": peer_id.as_str(),
                "track_id": track_id.as_str(),
            }),
        )
    }

    pub fn recording_started(
        session_id: &str,
        room_id: &RoomId,
        destination_path: &str,
    ) -> Self {
        Self::new(
            WebhookEventType::RecordingStarted,
            json!({
                "session_id": session_id,
                "room_id": room_id.as_str(),
                "destination_path": destination_path,
            }),
        )
    }

    pub fn recording_completed(
        session_id: &str,
        room_id: &RoomId,
        duration_secs: u64,
        bytes_captured: u64,
        packets_captured: u64,
    ) -> Self {
        Self::new(
            WebhookEventType::RecordingCompleted,
            json!({
                "session_id": session_id,
                "room_id": room_id.as_str(),
                "duration_secs": duration_secs,
                "bytes_captured": bytes_captured,
                "packets_captured": packets_captured,
            }),
        )
    }

    pub fn egress_failed(session_id: &str, room_id: &RoomId, reason: &str) -> Self {
        Self::new(
            WebhookEventType::EgressFailed,
            json!({
                "session_id": session_id,
                "room_id": room_id.as_str(),
                "reason": reason,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_serialization() {
        let room_id = RoomId::from("conf-room-1");
        let event = WebhookEvent::room_created(&room_id);

        let json_str = serde_json::to_string(&event).unwrap();
        assert!(json_str.contains("room_created"));
        assert!(json_str.contains("conf-room-1"));

        let parsed: WebhookEvent = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed.id, event.id);
        assert_eq!(parsed.event, WebhookEventType::RoomCreated);
        assert_eq!(parsed.payload["room_id"], "conf-room-1");
    }
}
