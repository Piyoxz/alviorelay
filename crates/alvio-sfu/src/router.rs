use crate::feedback::{KeyframeController, KeyframeKind};
use crate::nack::NackBuffer;
use crate::track::{StreamConsumer, StreamSource};
use alvio_core::StreamLayer;
use alvio_webrtc::AlvioRtpPacket;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

/// Media Hot-Path RTP Forwarding Router with Simulcast support.
///
/// Implements lock-free / low-contention lookups mapping publisher SSRC to active consumers,
/// caching packets in NackBuffer for packet-loss recovery, gating multi-layer Simulcast streams,
/// and rate-limiting keyframe requests.
pub struct RtpRouter {
    sources: DashMap<u32, Arc<StreamSource>>,
    consumers: DashMap<u32, Vec<Arc<StreamConsumer>>>,
    ssrc_to_primary: DashMap<u32, u32>,
    nack_buffers: DashMap<u32, Arc<NackBuffer>>,
    keyframe_controller: KeyframeController,
}

impl RtpRouter {
    pub fn new() -> Self {
        Self {
            sources: DashMap::new(),
            consumers: DashMap::new(),
            ssrc_to_primary: DashMap::new(),
            nack_buffers: DashMap::new(),
            keyframe_controller: KeyframeController::default(),
        }
    }

    /// Registers a new published source and initializes its packet cache ring buffers.
    ///
    /// If the source has multiple layers (Simulcast), registers all layer SSRCs.
    pub fn register_source(&self, source: Arc<StreamSource>) {
        debug!(ssrc = source.ssrc, track = %source.track_id, layers = source.layers.len(), "Registered RTP stream source");
        self.ssrc_to_primary.insert(source.ssrc, source.ssrc);
        self.nack_buffers
            .insert(source.ssrc, Arc::new(NackBuffer::default()));

        for (_layer, layer_ssrc) in &source.layers {
            self.ssrc_to_primary.insert(*layer_ssrc, source.ssrc);
            self.nack_buffers
                .insert(*layer_ssrc, Arc::new(NackBuffer::default()));
        }

        self.sources.insert(source.ssrc, source);
    }

    /// Unregisters an active stream source and drops its consumers, cache, and state.
    pub fn unregister_source(&self, source_ssrc: u32) {
        if let Some((_, source)) = self.sources.remove(&source_ssrc) {
            for (_layer, layer_ssrc) in &source.layers {
                self.ssrc_to_primary.remove(layer_ssrc);
                self.nack_buffers.remove(layer_ssrc);
            }
        }
        self.ssrc_to_primary.remove(&source_ssrc);
        self.consumers.remove(&source_ssrc);
        self.nack_buffers.remove(&source_ssrc);
        self.keyframe_controller.reset(source_ssrc);
    }

    /// Attaches a new consumer subscription to a stream source.
    pub fn add_consumer(&self, source_ssrc: u32, consumer: Arc<StreamConsumer>) {
        let primary_ssrc = self
            .ssrc_to_primary
            .get(&source_ssrc)
            .map(|r| *r)
            .unwrap_or(source_ssrc);
        debug!(
            primary_ssrc,
            target_ssrc = consumer.target_ssrc,
            consumer_id = %consumer.consumer_id,
            "Added RTP stream consumer"
        );
        self.consumers
            .entry(primary_ssrc)
            .or_default()
            .push(consumer);
    }

    /// Removes a consumer subscription.
    pub fn remove_consumer(&self, source_ssrc: u32, consumer_id: &str) {
        let primary_ssrc = self
            .ssrc_to_primary
            .get(&source_ssrc)
            .map(|r| *r)
            .unwrap_or(source_ssrc);
        if let Some(mut list) = self.consumers.get_mut(&primary_ssrc) {
            list.retain(|c| c.consumer_id != consumer_id);
        }
    }

    /// Requests a dynamic Simulcast layer switch for a specific consumer.
    pub fn switch_consumer_layer(
        &self,
        source_ssrc: u32,
        consumer_id: &str,
        target_layer: StreamLayer,
    ) -> bool {
        let primary_ssrc = self
            .ssrc_to_primary
            .get(&source_ssrc)
            .map(|r| *r)
            .unwrap_or(source_ssrc);
        if let Some(consumer_list) = self.consumers.get(&primary_ssrc) {
            for consumer in consumer_list.iter() {
                if consumer.consumer_id == consumer_id {
                    consumer.request_layer_switch(target_layer);
                    return true;
                }
            }
        }
        false
    }

    /// Routes an incoming RTP packet on the hot path.
    pub fn route_packet(&self, packet: &AlvioRtpPacket) -> Vec<AlvioRtpPacket> {
        self.route_packet_with_keyframe(packet, false)
    }

    /// Routes an incoming RTP packet with explicit keyframe signaling for synchronized layer switching.
    pub fn route_packet_with_keyframe(
        &self,
        packet: &AlvioRtpPacket,
        is_keyframe: bool,
    ) -> Vec<AlvioRtpPacket> {
        let packet_ssrc = packet.header.ssrc;

        if let Some(buffer) = self.nack_buffers.get(&packet_ssrc) {
            buffer.put(packet.clone());
        }

        let primary_ssrc = self
            .ssrc_to_primary
            .get(&packet_ssrc)
            .map(|r| *r)
            .unwrap_or(packet_ssrc);

        let packet_layer = if let Some(source) = self.sources.get(&primary_ssrc) {
            source
                .layer_for_ssrc(packet_ssrc)
                .unwrap_or(StreamLayer::High)
        } else {
            StreamLayer::High
        };

        let mut forwarded = Vec::new();
        if let Some(consumer_list) = self.consumers.get(&primary_ssrc) {
            forwarded.reserve(consumer_list.len());
            for consumer in consumer_list.iter() {
                if consumer.should_forward(packet_layer, is_keyframe) {
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
        let primary_ssrc = self
            .ssrc_to_primary
            .get(&source_ssrc)
            .map(|r| *r)
            .unwrap_or(source_ssrc);
        self.keyframe_controller
            .request_keyframe(primary_ssrc, kind, now)
    }

    pub fn nack_buffer(&self, source_ssrc: u32) -> Option<Arc<NackBuffer>> {
        self.nack_buffers.get(&source_ssrc).map(|r| Arc::clone(&r))
    }

    pub fn active_source_count(&self) -> usize {
        self.sources.len()
    }

    pub fn consumer_count(&self, source_ssrc: u32) -> usize {
        let primary_ssrc = self
            .ssrc_to_primary
            .get(&source_ssrc)
            .map(|r| *r)
            .unwrap_or(source_ssrc);
        self.consumers.get(&primary_ssrc).map_or(0, |c| c.len())
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
            0x80, 0x60, 0x00, 0x01, 0x00, 0x01, 0x5F, 0x90, 0x11, 0x11, 0x22,
            0x22, // Alice SSRC
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
            0x80, 0x60, 0x00, 0x02, 0x00, 0x01, 0x60, 0x00, 0x11, 0x11, 0x22, 0x22, 0xEE, 0xFF,
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
        assert!(!router.request_keyframe(
            source_ssrc,
            KeyframeKind::Pli,
            now + Duration::from_millis(50)
        ));
    }
}
