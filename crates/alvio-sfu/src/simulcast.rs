use alvio_core::{PeerId, StreamLayer, TrackId};
use std::collections::HashMap;

/// A multi-layer Simulcast video track originating from a single publisher.
#[derive(Debug, Clone)]
pub struct SimulcastSource {
    pub track_id: TrackId,
    pub publisher_peer_id: PeerId,
    pub layers: HashMap<StreamLayer, u32>,
    pub ssrc_to_layer: HashMap<u32, StreamLayer>,
}

impl SimulcastSource {
    pub fn new(track_id: TrackId, publisher_peer_id: PeerId) -> Self {
        Self {
            track_id,
            publisher_peer_id,
            layers: HashMap::new(),
            ssrc_to_layer: HashMap::new(),
        }
    }

    /// Add a layer with its associated SSRC.
    pub fn add_layer(&mut self, layer: StreamLayer, ssrc: u32) {
        self.layers.insert(layer, ssrc);
        self.ssrc_to_layer.insert(ssrc, layer);
    }

    pub fn ssrc_for_layer(&self, layer: StreamLayer) -> Option<u32> {
        self.layers.get(&layer).copied()
    }

    pub fn layer_for_ssrc(&self, ssrc: u32) -> Option<StreamLayer> {
        self.ssrc_to_layer.get(&ssrc).copied()
    }

    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

/// Adaptive Simulcast Layer Selector based on bandwidth estimations and consumer capabilities.
pub struct LayerSelector {
    low_bitrate_threshold: u64,
    mid_bitrate_threshold: u64,
}

impl LayerSelector {
    pub const DEFAULT_LOW_THRESHOLD_BPS: u64 = 300_000;   // 300 kbps
    pub const DEFAULT_MID_THRESHOLD_BPS: u64 = 1_000_000; // 1 Mbps

    pub fn new(low_threshold: u64, mid_threshold: u64) -> Self {
        Self {
            low_bitrate_threshold: low_threshold,
            mid_bitrate_threshold: mid_threshold,
        }
    }

    /// Determines the optimal layer given the subscriber's measured available bandwidth.
    pub fn select_layer(&self, available_bitrate_bps: u64) -> StreamLayer {
        if available_bitrate_bps < self.low_bitrate_threshold {
            StreamLayer::Low
        } else if available_bitrate_bps < self.mid_bitrate_threshold {
            StreamLayer::Medium
        } else {
            StreamLayer::High
        }
    }
}

impl Default for LayerSelector {
    fn default() -> Self {
        Self::new(Self::DEFAULT_LOW_THRESHOLD_BPS, Self::DEFAULT_MID_THRESHOLD_BPS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulcast_source_layer_mapping() {
        let mut sim = SimulcastSource::new(TrackId::from("cam-alice"), PeerId::from("alice"));
        sim.add_layer(StreamLayer::Low, 1001);
        sim.add_layer(StreamLayer::Medium, 1002);
        sim.add_layer(StreamLayer::High, 1003);

        assert_eq!(sim.ssrc_for_layer(StreamLayer::Low), Some(1001));
        assert_eq!(sim.ssrc_for_layer(StreamLayer::Medium), Some(1002));
        assert_eq!(sim.ssrc_for_layer(StreamLayer::High), Some(1003));

        assert_eq!(sim.layer_for_ssrc(1001), Some(StreamLayer::Low));
        assert_eq!(sim.layer_for_ssrc(1002), Some(StreamLayer::Medium));
        assert_eq!(sim.layer_for_ssrc(1003), Some(StreamLayer::High));
        assert_eq!(sim.layer_for_ssrc(9999), None);
    }

    #[test]
    fn test_layer_selector_thresholds() {
        let selector = LayerSelector::default();

        assert_eq!(selector.select_layer(150_000), StreamLayer::Low);
        assert_eq!(selector.select_layer(600_000), StreamLayer::Medium);
        assert_eq!(selector.select_layer(2_000_000), StreamLayer::High);
    }
}
