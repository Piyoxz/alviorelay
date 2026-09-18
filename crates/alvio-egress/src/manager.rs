use crate::error::{EgressError, EgressResult};
use crate::session::{current_timestamp, RecordingConfig, RecordingOutput, RecordingStatus};
use crate::tap::MediaTap;
use alvio_core::RoomId;
use alvio_storage::StorageBackend;
use alvio_webrtc::AlvioRtpPacket;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

struct ActiveSession {
    config: RecordingConfig,
    tap: Arc<MediaTap>,
    receiver: Mutex<Option<mpsc::Receiver<AlvioRtpPacket>>>,
    status: Arc<RwLock<RecordingStatus>>,
    started_at: u64,
}

/// Central Egress and Recording Supervisor managing active room recording sessions.
pub struct RecordingManager {
    sessions: DashMap<String, ActiveSession>,
    room_to_session: DashMap<RoomId, String>,
    storage: Arc<dyn StorageBackend>,
}

impl RecordingManager {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self {
            sessions: DashMap::new(),
            room_to_session: DashMap::new(),
            storage,
        }
    }

    /// Starts an active recording session for a room.
    ///
    /// Returns the assigned `session_id` and the `MediaTap` handle to attach to the SFU router.
    pub fn start_recording(
        &self,
        config: RecordingConfig,
    ) -> EgressResult<(String, Arc<MediaTap>)> {
        let room_id = config.room_id.clone();

        if self.room_to_session.contains_key(&room_id) {
            return Err(EgressError::SessionAlreadyActive(room_id.to_string()));
        }

        let session_id = format!("rec_{}", Uuid::new_v4().simple());
        let started_at = current_timestamp();

        let (tap, rx) = MediaTap::new(MediaTap::DEFAULT_QUEUE_CAPACITY);
        let status = Arc::new(RwLock::new(RecordingStatus::Active { started_at }));

        let active_session = ActiveSession {
            config: config.clone(),
            tap: Arc::clone(&tap),
            receiver: Mutex::new(Some(rx)),
            status,
            started_at,
        };

        self.sessions.insert(session_id.clone(), active_session);
        self.room_to_session
            .insert(room_id.clone(), session_id.clone());

        info!(
            %session_id,
            %room_id,
            format = ?config.format,
            dest = %config.destination_path,
            "Started media recording session"
        );

        Ok((session_id, tap))
    }

    /// Stops an ongoing recording session and computes the final recording metadata.
    pub fn stop_recording(&self, session_id: &str) -> EgressResult<RecordingOutput> {
        let Some((_, session)) = self.sessions.remove(session_id) else {
            return Err(EgressError::SessionNotFound(session_id.to_string()));
        };

        self.room_to_session.remove(&session.config.room_id);

        // Close the media tap
        session.tap.close();

        let ended_at = current_timestamp();
        let duration_secs = ended_at.saturating_sub(session.started_at);

        *session.status.write() = RecordingStatus::Completed {
            started_at: session.started_at,
            ended_at,
            duration_secs,
            destination_path: session.config.destination_path.clone(),
        };

        let output = RecordingOutput {
            session_id: session_id.to_string(),
            room_id: session.config.room_id,
            format: session.config.format,
            destination_path: session.config.destination_path,
            duration_secs,
            bytes_captured: session.tap.bytes_captured(),
            packets_captured: session.tap.packets_captured(),
        };

        info!(
            %session_id,
            duration_secs = output.duration_secs,
            bytes = output.bytes_captured,
            packets = output.packets_captured,
            "Stopped and finalized recording session"
        );

        Ok(output)
    }

    /// Queries the current lifecycle status of a session.
    pub fn status(&self, session_id: &str) -> Option<RecordingStatus> {
        self.sessions
            .get(session_id)
            .map(|s| s.status.read().clone())
    }

    /// Checks if a room currently has an active recording session.
    pub fn is_room_recording(&self, room_id: &RoomId) -> bool {
        self.room_to_session.contains_key(room_id)
    }

    /// Extracts the packet receiver stream for asynchronous process piping (e.g. FFmpeg supervisor).
    pub fn take_receiver(&self, session_id: &str) -> Option<mpsc::Receiver<AlvioRtpPacket>> {
        self.sessions
            .get(session_id)
            .and_then(|s| s.receiver.lock().take())
    }

    pub fn storage(&self) -> &Arc<dyn StorageBackend> {
        &self.storage
    }

    pub fn active_session_count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alvio_storage::LocalStorage;
    use alvio_webrtc::{AlvioRtpPacket, RtpHeader};
    use bytes::Bytes;

    fn make_packet(seq: u16) -> AlvioRtpPacket {
        AlvioRtpPacket {
            header: RtpHeader {
                version: 2,
                has_padding: false,
                has_extension: false,
                csrc_count: 0,
                marker: false,
                payload_type: 96,
                sequence_number: seq,
                timestamp: 1000,
                ssrc: 999,
            },
            payload: Bytes::from_static(b"packet-audio-data"),
        }
    }

    #[test]
    fn test_recording_manager_lifecycle() {
        let tmp_dir = std::env::temp_dir().join(format!(
            "alvio_rec_mgr_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let storage = Arc::new(LocalStorage::new(&tmp_dir));
        let manager = RecordingManager::new(storage);

        let room_id = RoomId::from("room-conference-1");
        let config = RecordingConfig::mp4(room_id.clone(), "conference-1.mp4");

        // Start recording
        let (session_id, tap) = manager.start_recording(config.clone()).unwrap();
        assert!(manager.is_room_recording(&room_id));
        assert_eq!(manager.active_session_count(), 1);

        // Cannot start duplicate session for the same room
        let dup_err = manager.start_recording(config);
        assert!(dup_err.is_err());

        // Stream some packets to tap
        for i in 1..=10 {
            tap.push_packet(make_packet(i));
        }

        // Check active status
        assert!(matches!(
            manager.status(&session_id),
            Some(RecordingStatus::Active { .. })
        ));

        // Stop recording
        let output = manager.stop_recording(&session_id).unwrap();
        assert_eq!(output.session_id, session_id);
        assert_eq!(output.room_id, room_id);
        assert_eq!(output.packets_captured, 10);
        assert_eq!(output.bytes_captured, 10 * 17);

        assert!(!manager.is_room_recording(&room_id));
        assert_eq!(manager.active_session_count(), 0);
    }
}
