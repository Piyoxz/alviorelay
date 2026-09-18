use crate::feedback::{KeyframeController, KeyframeKind};
use crate::nack::NackBuffer;
use crate::track::{StreamConsumer, StreamSource};
use alvio_webrtc::AlvioRtpPacket;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

/// Media Hot-Path RTP Forwarding Router.
///
/// Implements lock-free / low-contention lookups mapping publisher SSRC to active consumers,
/// caching packets in NackBuffer for packet-loss recovery, and rate-limiting keyframe requests.
pub struct RtpRouter {
    sources: DashMap<u32, Arc<StreamSource>>,
    consumers: DashMap<u32, Vec<Arc<StreamConsumer>>>,
    nack_buffers: DashMap<u32, Arc<NackBuffer>>,
    keyframe_controller: KeyframeController,
}

impl RtpRouter {
    pub fn new() -> Self {
        Self {
            sources: DashMap::new(),
            consumers: DashMap::new(),
            nack_buffers: DashMap::new(),
            keyframe_controller: KeyframeController::default(),
        }
    }

    /// Registers a new published source and initializes its packet cache ring buffer.
    pub fn register_source(&self, source: Arc<StreamSource>) {
        debug!(ssrc = source.ssrc, track = %source.track_id, "Registered RTP stream source");
        self.nack_buffers
            .insert(source.ssrc, Arc::new(NackBuffer::default()));
        self.sources.insert(source.ssrc, source);
    }

    /// Unregisters an active stream source and drops its consumers, cache, and state.
    pub fn unregister_source(&self, source_ssrc: u32) {
        self.sources.remove(&source_ssrc);
        self.consumers.remove(&source_ssrc);
        self.nack_buffers.remove(&source_ssrc);
        self.keyframe_controller.reset(source_ssrc);
    }

    /// Attaches a new consumer subscription to a stream source.
    pub fn add_consumer(&self, source_ssrc: u32, consumer: Arc<StreamConsumer>) {
        debug!(
            source_ssrc,
            target_ssrc = consumer.target_ssrc,
            consumer_id = %consumer.consumer_id,
            "Added RTP stream consumer"
        );
        self.consumers
            .entry(source_ssrc)
            .or_default()
            .push(consumer);
    }

    /// Removes a consumer subscription.
    pub fn remove_consumer(&self, source_ssrc: u32, consumer_id: &str) {
        if let Some(mut list) = self.consumers.get_mut(&source_ssrc) {
            list.retain(|c| c.consumer_id != consumer_id);
        }
    }

    /// Routes an incoming RTP packet on the hot path.
    ///
    /// Stores the packet into the source's NackBuffer ring buffer,
    /// and for every active consumer, rewrites the packet with continuous
    /// sequence numbering and the subscriber's negotiated SSRC.
    pub fn route_packet(&self, packet: &AlvioRtpPacket) -> Vec<AlvioRtpPacket> {
        let source_ssrc = packet.header.ssrc;

        // Cache packet in NACK ring buffer for instant local retransmission
        if let Some(buffer) = self.nack_buffers.get(&source_ssrc) {
            buffer.put(packet.clone());
        }

        let mut forwarded = Vec::new();
        if let Some(consumer_list) = self.consumers.get(&source_ssrc) {
            forwarded.reserve(consumer_list.len());
            for consumer in consumer_list.iter() {
                if consumer.is_active() {
                    let rewritten = consumer.rewrite_packet(packet);
                    forwarded.push(rewritten);
                }
            }
        }

        forwarded
    }

    /// Handle NACK retransmission request: retrieves cached packets from the source ring buffer.
    pub fn handle_nack(&self, source_ssrc: u32, requested_seqs: &[u16]) -> Vec<AlvioRtpPacket> {
        if let Some(buffer) = self.nack_buffers.get(&source_ssrc) {
            buffer.get_batch(requested_seqs)
        } else {
            Vec::new()
        }
    }

    /// Evaluates keyframe request (PLI/FIR) with rate limiting against keyframe storms.
    pub fn request_keyframe(&self, source_ssrc: u32, kind: KeyframeKind, now: Instant) -> bool {
        self.keyframe_controller.request_keyframe(source_ssrc, kind, now)
    }

    pub fn nack_buffer(&self, source_ssrc: u32) -> Option<Arc<NackBuffer>> {
        self.nack_buffers.get(&source_ssrc).map(|r| Arc::clone(&r))
    }

    pub fn active_source_count(&self) -> usize {
        self.sources.len()
    }

    pub fn consumer_count(&self, source_ssrc: u32) -> usize {
        self.consumers.get(&source_ssrc).map_or(0, |c| c.len())
    }
}

impl Default for RtpRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
    use bytes::Bytes;
    use std::time::Duration;

    #[test]
    fn test_rtp_fanout_routing_and_nack_cache() {
        let router = RtpRouter::new();

        let source_ssrc = 0x11112222;
        let source = Arc::new(StreamSource::new(
            TrackId::from("track-video-alice"),
            PeerId::from("alice"),
            source_ssrc,
            StreamKind::Video,
            90000,
        ));
        router.register_source(source);

        // Add 2 consumers (Bob and Charlie)
        let consumer_bob = Arc::new(StreamConsumer::new(
            "sub-bob".to_string(),
            PeerId::from("bob"),
            0x33334444, // Bob's negotiated SSRC
            100,        // Initial sequence number for Bob
            StreamLayer::High,
        ));
        let consumer_charlie = Arc::new(StreamConsumer::new(
            "sub-charlie".to_string(),
            PeerId::from("charlie"),
            0x55556666, // Charlie's negotiated SSRC
            500,        // Initial sequence number for Charlie
            StreamLayer::High,
        ));

        router.add_consumer(source_ssrc, consumer_bob);
        router.add_consumer(source_ssrc, consumer_charlie);

        assert_eq!(router.consumer_count(source_ssrc), 2);

        // Simulate incoming packet from Alice (sequence number 1)
        let raw_packet = vec![
            0x80, 0x60, 0x00, 0x01,
            0x00, 0x01, 0x5F, 0x90,
            0x11, 0x11, 0x22, 0x22, // Alice SSRC
            0xAA, 0xBB, 0xCC, 0xDD,
        ];
        let packet = AlvioRtpPacket::parse(Bytes::from(raw_packet)).unwrap();

        // Forward through router
        let routed = router.route_packet(&packet);
        assert_eq!(routed.len(), 2);

        // Bob's packet check
        assert_eq!(routed[0].header.ssrc, 0x33334444);
        assert_eq!(routed[0].header.sequence_number, 100);
        assert_eq!(routed[0].payload.as_ref(), &[0xAA, 0xBB, 0xCC, 0xDD]);

        // Charlie's packet check
        assert_eq!(routed[1].header.ssrc, 0x55556666);
        assert_eq!(routed[1].header.sequence_number, 500);
        assert_eq!(routed[1].payload.as_ref(), &[0xAA, 0xBB, 0xCC, 0xDD]);

        // Next packet from Alice (sequence number 2)
        let raw_packet2 = vec![
            0x80, 0x60, 0x00, 0x02,
            0x00, 0x01, 0x60, 0x00,
            0x11, 0x11, 0x22, 0x22,
            0xEE, 0xFF,
        ];
        let packet2 = AlvioRtpPacket::parse(Bytes::from(raw_packet2)).unwrap();
        let routed2 = router.route_packet(&packet2);

        // Monotonic increment for both subscribers
        assert_eq!(routed2[0].header.sequence_number, 101);
        assert_eq!(routed2[1].header.sequence_number, 501);

        // Test NACK handling from source's NackBuffer:
        // Request retransmission of packet sequence 1 and 2
        let retransmitted = router.handle_nack(source_ssrc, &[1, 2, 99]);
        assert_eq!(retransmitted.len(), 2);
        assert_eq!(retransmitted[0].header.sequence_number, 1);
        assert_eq!(retransmitted[1].header.sequence_number, 2);

        // Test Keyframe Rate Limiting via Router
        let now = Instant::now();
        assert!(router.request_keyframe(source_ssrc, KeyframeKind::Pli, now));
        // Second immediate call throttled
        assert!(!router.request_keyframe(source_ssrc, KeyframeKind::Pli, now + Duration::from_millis(50)));
    }
}
