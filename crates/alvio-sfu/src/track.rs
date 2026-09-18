use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
use alvio_webrtc::AlvioRtpPacket;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

/// A published media stream originating from a Peer.
#[derive(Debug, Clone)]
pub struct StreamSource {
    pub track_id: TrackId,
    pub publisher_peer_id: PeerId,
    pub ssrc: u32,
    pub kind: StreamKind,
    pub clock_rate: u32,
    pub layers: Vec<(StreamLayer, u32)>,
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
            layers: vec![(StreamLayer::High, ssrc)],
        }
    }

    pub fn with_layers(
        track_id: TrackId,
        publisher_peer_id: PeerId,
        primary_ssrc: u32,
        kind: StreamKind,
        clock_rate: u32,
        layers: Vec<(StreamLayer, u32)>,
    ) -> Self {
        Self {
            track_id,
            publisher_peer_id,
            ssrc: primary_ssrc,
            kind,
            clock_rate,
            layers,
        }
    }

    pub fn add_layer(&mut self, layer: StreamLayer, ssrc: u32) {
        self.layers.retain(|(l, _)| *l != layer);
        self.layers.push((layer, ssrc));
    }

    pub fn layer_for_ssrc(&self, ssrc: u32) -> Option<StreamLayer> {
        for (l, s) in &self.layers {
            if *s == ssrc {
                return Some(*l);
            }
        }
        if self.ssrc == ssrc {
            Some(StreamLayer::High)
        } else {
            None
        }
    }
}

/// A subscriber consumption channel for a `StreamSource` with dynamic Simulcast layer switching.
pub struct StreamConsumer {
    pub consumer_id: String,
    pub subscriber_peer_id: PeerId,
    pub target_ssrc: u32,
    target_layer: RwLock<StreamLayer>,
    current_layer: RwLock<StreamLayer>,
    pending_switch: RwLock<Option<StreamLayer>>,
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
            target_layer: RwLock::new(target_layer),
            current_layer: RwLock::new(target_layer),
            pending_switch: RwLock::new(None),
            seq_counter: AtomicU16::new(initial_seq),
            active: AtomicBool::new(true),
        }
    }

    pub fn target_layer(&self) -> StreamLayer {
        *self.target_layer.read()
    }

    pub fn current_layer(&self) -> StreamLayer {
        *self.current_layer.read()
    }

    /// Request a switch to a new layer.
    ///
    /// The switch is queued until a keyframe on the target layer arrives,
    /// preventing visual glitches or decoder stalls for the subscriber.
    pub fn request_layer_switch(&self, new_layer: StreamLayer) {
        if new_layer != self.current_layer() {
            *self.target_layer.write() = new_layer;
            *self.pending_switch.write() = Some(new_layer);
        } else {
            *self.pending_switch.write() = None;
        }
    }

    /// Determines whether an incoming packet for `packet_layer` should be forwarded.
    ///
    /// If a switch is pending, waits for a keyframe on `target_layer`.
    /// Until that keyframe arrives, continues forwarding the `current_layer`.
    pub fn should_forward(&self, packet_layer: StreamLayer, is_keyframe: bool) -> bool {
        if !self.is_active() {
            return false;
        }

        let pending = *self.pending_switch.read();
        if let Some(target) = pending {
            if packet_layer == target {
                if is_keyframe {
                    // Clean keyframe synchronization achieved: complete the layer switch!
                    *self.current_layer.write() = target;
                    *self.pending_switch.write() = None;
                    return true;
                }
                // Target layer packet received, but not a keyframe yet -> hold off
                return false;
            } else if packet_layer == *self.current_layer.read() {
                // Continue forwarding old layer until target keyframe arrives
                return true;
            } else {
                return false;
            }
        }

        packet_layer == *self.current_layer.read()
    }

    /// Rewrites the incoming packet into a continuous subscriber-specific sequence.
    ///
    /// Monotonically increments `seq_counter` regardless of layer transitions.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_consumer_simulcast_layer_switching() {
        let consumer = StreamConsumer::new(
            "sub-1".to_string(),
            PeerId::from("bob"),
            5000,
            10,
            StreamLayer::High,
        );

        assert_eq!(consumer.current_layer(), StreamLayer::High);
        assert_eq!(consumer.target_layer(), StreamLayer::High);

        // While on High, only High packets are forwarded
        assert!(consumer.should_forward(StreamLayer::High, false));
        assert!(!consumer.should_forward(StreamLayer::Low, false));
        assert!(!consumer.should_forward(StreamLayer::Medium, false));

        // Request switch to Low
        consumer.request_layer_switch(StreamLayer::Low);
        assert_eq!(consumer.target_layer(), StreamLayer::Low);
        assert_eq!(consumer.current_layer(), StreamLayer::High); // Still on High until Low keyframe arrives

        // High packets are still forwarded
        assert!(consumer.should_forward(StreamLayer::High, false));

        // Low non-keyframe packet is dropped
        assert!(!consumer.should_forward(StreamLayer::Low, false));

        // Low keyframe arrives! Switch completed!
        assert!(consumer.should_forward(StreamLayer::Low, true));
        assert_eq!(consumer.current_layer(), StreamLayer::Low);

        // Now on Low, High packets are dropped and Low packets are forwarded
        assert!(!consumer.should_forward(StreamLayer::High, false));
        assert!(consumer.should_forward(StreamLayer::Low, false));
    }
}
