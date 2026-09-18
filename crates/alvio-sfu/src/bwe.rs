use alvio_core::StreamLayer;
use parking_lot::RwLock;

/// Bandwidth and Congestion Controller tracking transport-wide estimates.
pub struct BweController {
    state: RwLock<BweState>,
}

#[derive(Debug, Clone)]
pub struct BweState {
    pub available_bitrate_bps: u64,
    pub loss_ratio: f32,
    pub rtt_ms: u32,
}

impl BweController {
    pub const DEFAULT_INITIAL_BITRATE: u64 = 1_500_000; // 1.5 Mbps

    pub fn new(initial_bitrate_bps: u64) -> Self {
        Self {
            state: RwLock::new(BweState {
                available_bitrate_bps: initial_bitrate_bps,
                loss_ratio: 0.0,
                rtt_ms: 0,
            }),
        }
    }

    /// Update the estimated bandwidth, loss ratio, and RTT.
    pub fn update_estimate(&self, bitrate_bps: u64, loss_ratio: f32, rtt_ms: u32) {
        let mut state = self.state.write();
        state.available_bitrate_bps = bitrate_bps;
        state.loss_ratio = loss_ratio;
        state.rtt_ms = rtt_ms;
    }

    pub fn current_state(&self) -> BweState {
        self.state.read().clone()
    }

    /// Recommends a stream layer based on available bandwidth thresholds.
    ///
    /// - Below `low_threshold_bps`: `StreamLayer::Low`
    /// - Between `low_threshold_bps` and `mid_threshold_bps`: `StreamLayer::Medium`
    /// - At or above `mid_threshold_bps`: `StreamLayer::High`
    pub fn recommend_layer(&self, low_threshold_bps: u64, mid_threshold_bps: u64) -> StreamLayer {
        let state = self.state.read();
        if state.available_bitrate_bps < low_threshold_bps {
            StreamLayer::Low
        } else if state.available_bitrate_bps < mid_threshold_bps {
            StreamLayer::Medium
        } else {
            StreamLayer::High
        }
    }
}

impl Default for BweController {
    fn default() -> Self {
        Self::new(Self::DEFAULT_INITIAL_BITRATE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bwe_recommend_layer() {
        let bwe = BweController::new(2_000_000);

        // At 2 Mbps, with thresholds (300k, 1M), should recommend High
        assert_eq!(bwe.recommend_layer(300_000, 1_000_000), StreamLayer::High);

        // Degrade bandwidth to 800 kbps -> Medium
        bwe.update_estimate(800_000, 0.02, 50);
        assert_eq!(bwe.recommend_layer(300_000, 1_000_000), StreamLayer::Medium);

        // Degrade bandwidth to 200 kbps -> Low
        bwe.update_estimate(200_000, 0.15, 120);
        assert_eq!(bwe.recommend_layer(300_000, 1_000_000), StreamLayer::Low);
    }
}
