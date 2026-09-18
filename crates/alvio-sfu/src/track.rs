use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
use alvio_webrtc::AlvioRtpPacket;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

/// A published media stream originating from a Peer.
#[derive(Debug)]
pub struct StreamSource {
    pub track_id: TrackId,
    pub publisher_peer_id: PeerId,
    pub ssrc: u32,
    pub kind: StreamKind,
    pub clock_rate: u32,
}

impl StreamSource {
    pub fn new(
        track_id: TrackId,
        publisher_peer_id: PeerId,
        ssrc: u32,
        kind: StreamKind,
        clock_rate: u32,
    ) -> Self {
        Self {
            track_id,
            publisher_peer_id,
            ssrc,
            kind,
            clock_rate,
        }
    }
}

/// A subscriber consumption channel for a `StreamSource`.
pub struct StreamConsumer {
    pub consumer_id: String,
    pub subscriber_peer_id: PeerId,
    pub target_ssrc: u32,
    pub target_layer: StreamLayer,
    seq_counter: AtomicU16,
    active: AtomicBool,
}

impl StreamConsumer {
    pub fn new(
        consumer_id: String,
        subscriber_peer_id: PeerId,
        target_ssrc: u32,
        initial_seq: u16,
        target_layer: StreamLayer,
    ) -> Self {
        Self {
            consumer_id,
            subscriber_peer_id,
            target_ssrc,
            target_layer,
            seq_counter: AtomicU16::new(initial_seq),
            active: AtomicBool::new(true),
        }
    }

    /// Rewrites the incoming packet into a continuous subscriber-specific sequence.
    pub fn rewrite_packet(&self, packet: &AlvioRtpPacket) -> AlvioRtpPacket {
        let seq = self.seq_counter.fetch_add(1, Ordering::Relaxed);
        packet.with_rewritten_meta(self.target_ssrc, seq, packet.header.timestamp)
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Relaxed)
    }

    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Relaxed);
    }
}
