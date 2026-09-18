use crate::track::{StreamConsumer, StreamSource};
use alvio_webrtc::AlvioRtpPacket;
use dashmap::DashMap;
use std::sync::Arc;
use tracing::debug;

/// Media Hot-Path RTP Forwarding Router.
///
/// Implements lock-free / low-contention lookups mapping publisher SSRC to active consumers,
/// rewriting SSRC and sequence numbers on the fly with zero blocking allocations.
pub struct RtpRouter {
    sources: DashMap<u32, Arc<StreamSource>>,
    consumers: DashMap<u32, Vec<Arc<StreamConsumer>>>,
}

impl RtpRouter {
    pub fn new() -> Self {
        Self {
            sources: DashMap::new(),
            consumers: DashMap::new(),
        }
    }

    /// Registers a new published source.
    pub fn register_source(&self, source: Arc<StreamSource>) {
        debug!(ssrc = source.ssrc, track = %source.track_id, "Registered RTP stream source");
        self.sources.insert(source.ssrc, source);
    }

    /// Unregisters an active stream source and drops its consumers.
    pub fn unregister_source(&self, source_ssrc: u32) {
        self.sources.remove(&source_ssrc);
        self.consumers.remove(&source_ssrc);
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
    /// For every active consumer, rewrites the packet with continuous sequence numbering
    /// and the subscriber's negotiated SSRC.
    pub fn route_packet(&self, packet: &AlvioRtpPacket) -> Vec<AlvioRtpPacket> {
        let source_ssrc = packet.header.ssrc;
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

    #[test]
    fn test_rtp_fanout_routing() {
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
    }
}
