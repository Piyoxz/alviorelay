use crate::supervisor::OutputFormat;
use alvio_core::RoomId;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Configuration for initiating an egress recording session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingConfig {
    pub room_id: RoomId,
    pub format: OutputFormat,
    pub destination_path: String,
    pub record_audio: bool,
    pub record_video: bool,
}

impl RecordingConfig {
    pub fn mp4(room_id: impl Into<RoomId>, destination_path: impl Into<String>) -> Self {
        Self {
            room_id: room_id.into(),
            format: OutputFormat::Mp4,
            destination_path: destination_path.into(),
            record_audio: true,
            record_video: true,
        }
    }

    pub fn webm(room_id: impl Into<RoomId>, destination_path: impl Into<String>) -> Self {
        Self {
            room_id: room_id.into(),
            format: OutputFormat::WebM,
            destination_path: destination_path.into(),
            record_audio: true,
            record_video: true,
        }
    }
}

/// Lifecycle state of an active or completed recording session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingStatus {
    Starting,
    Active {
        started_at: u64,
    },
    Stopping,
    Completed {
        started_at: u64,
        ended_at: u64,
        duration_secs: u64,
        destination_path: String,
    },
    Failed {
        reason: String,
    },
}

/// Final summary output of a completed recording session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingOutput {
    pub session_id: String,
    pub room_id: RoomId,
    pub format: OutputFormat,
    pub destination_path: String,
    pub duration_secs: u64,
    pub bytes_captured: u64,
    pub packets_captured: u64,
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
