use alvio_webrtc::AlvioRtpPacket;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::warn;

/// Virtual recording tap that captures raw media packets directly from the hot path.
pub struct MediaTap {
    tx: mpsc::Sender<AlvioRtpPacket>,
    packets_captured: AtomicU64,
    bytes_captured: AtomicU64,
    active: AtomicBool,
}

impl MediaTap {
    pub const DEFAULT_QUEUE_CAPACITY: usize = 2048;

    pub fn new(capacity: usize) -> (Arc<Self>, mpsc::Receiver<AlvioRtpPacket>) {
        let (tx, rx) = mpsc::channel(capacity);
        let tap = Arc::new(Self {
            tx,
            packets_captured: AtomicU64::new(0),
            bytes_captured: AtomicU64::new(0),
            active: AtomicBool::new(true),
        });
        (tap, rx)
    }

    /// Captures an incoming RTP packet out-of-band.
    ///
    /// Never blocks the caller; if queue is momentarily full, warns and drops to protect hot path latency.
    pub fn push_packet(&self, packet: AlvioRtpPacket) {
        if !self.active.load(Ordering::Relaxed) {
            return;
        }

        let payload_len = packet.payload.len() as u64;

        match self.tx.try_send(packet) {
            Ok(()) => {
                self.packets_captured.fetch_add(1, Ordering::Relaxed);
                self.bytes_captured
                    .fetch_add(payload_len, Ordering::Relaxed);
            }
            Err(mpsc::error::TrySendError::Full(_)) => {
                warn!("MediaTap bounded queue is full, dropping frame to preserve SFU real-time stability");
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.active.store(false, Ordering::Relaxed);
            }
        }
    }

    pub fn close(&self) {
        self.active.store(false, Ordering::Relaxed);
    }

    pub fn packets_captured(&self) -> u64 {
        self.packets_captured.load(Ordering::Relaxed)
    }

    pub fn bytes_captured(&self) -> u64 {
        self.bytes_captured.load(Ordering::Relaxed)
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }
}

impl Default for MediaTap {
    fn default() -> Self {
        let (tap, _rx) = Self::new(Self::DEFAULT_QUEUE_CAPACITY);
        // Note: standalone default is rarely used directly without rx
        match Arc::try_unwrap(tap) {
            Ok(tap) => tap,
            Err(arc) => (*arc).clone_box(),
        }
    }
}

impl MediaTap {
    fn clone_box(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            packets_captured: AtomicU64::new(self.packets_captured()),
            bytes_captured: AtomicU64::new(self.bytes_captured()),
            active: AtomicBool::new(self.is_active()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alvio_webrtc::RtpHeader;
    use bytes::Bytes;

    fn make_test_packet(seq: u16) -> AlvioRtpPacket {
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
                ssrc: 1234,
            },
            payload: Bytes::from_static(b"media-payload"),
        }
    }

    #[tokio::test]
    async fn test_media_tap_buffering() {
        let (tap, mut rx) = MediaTap::new(10);

        for i in 1..=5 {
            tap.push_packet(make_test_packet(i));
        }

        assert_eq!(tap.packets_captured(), 5);
        assert_eq!(tap.bytes_captured(), 5 * 13);

        for i in 1..=5 {
            let received = rx.recv().await.unwrap();
            assert_eq!(received.header.sequence_number, i);
        }
    }
}
