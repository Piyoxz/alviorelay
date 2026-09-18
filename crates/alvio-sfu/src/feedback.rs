use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::debug;

/// Kind of RTCP feedback intra-frame request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyframeKind {
    /// Picture Loss Indication (RFC 4585)
    Pli,
    /// Full Intra Request (RFC 5104)
    Fir,
}

/// Controller responsible for managing downstream keyframe requests (PLI / FIR)
/// and rate-limiting them to avoid "intra-frame storms" on publishers.
pub struct KeyframeController {
    cooldown: Duration,
    last_requests: RwLock<HashMap<u32, Instant>>,
}

impl KeyframeController {
    pub const DEFAULT_COOLDOWN_MS: u64 = 500;

    pub fn new(cooldown: Duration) -> Self {
        Self {
            cooldown,
            last_requests: RwLock::new(HashMap::new()),
        }
    }

    /// Evaluates a keyframe request for a stream SSRC.
    ///
    /// Returns `true` if the request is allowed through to the publisher,
    /// or `false` if suppressed due to cooldown throttling.
    pub fn request_keyframe(&self, ssrc: u32, kind: KeyframeKind, now: Instant) -> bool {
        let mut last_requests = self.last_requests.write();
        if let Some(&last_time) = last_requests.get(&ssrc) {
            if now.saturating_duration_since(last_time) < self.cooldown {
                debug!(ssrc, ?kind, "Suppressed keyframe request within cooldown window");
                return false;
            }
        }

        last_requests.insert(ssrc, now);
        debug!(ssrc, ?kind, "Dispatched keyframe request to stream source");
        true
    }

    pub fn reset(&self, ssrc: u32) {
        self.last_requests.write().remove(&ssrc);
    }

    pub fn cooldown(&self) -> Duration {
        self.cooldown
    }
}

impl Default for KeyframeController {
    fn default() -> Self {
        Self::new(Duration::from_millis(Self::DEFAULT_COOLDOWN_MS))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_cooldown_throttling() {
        let controller = KeyframeController::new(Duration::from_millis(200));
        let now = Instant::now();
        let ssrc = 0x11223344;

        // First request is allowed
        assert!(controller.request_keyframe(ssrc, KeyframeKind::Pli, now));

        // Rapid second request (at +50ms) is throttled
        assert!(!controller.request_keyframe(ssrc, KeyframeKind::Pli, now + Duration::from_millis(50)));

        // Rapid FIR request is also throttled
        assert!(!controller.request_keyframe(ssrc, KeyframeKind::Fir, now + Duration::from_millis(150)));

        // After cooldown (+250ms), request is allowed again
        assert!(controller.request_keyframe(ssrc, KeyframeKind::Pli, now + Duration::from_millis(250)));
    }

    #[test]
    fn test_independent_ssrc_keyframe_requests() {
        let controller = KeyframeController::new(Duration::from_millis(200));
        let now = Instant::now();

        // Requests for different SSRCs should not block each other
        assert!(controller.request_keyframe(1001, KeyframeKind::Pli, now));
        assert!(controller.request_keyframe(1002, KeyframeKind::Pli, now));

        // Both are in cooldown
        assert!(!controller.request_keyframe(1001, KeyframeKind::Pli, now + Duration::from_millis(50)));
        assert!(!controller.request_keyframe(1002, KeyframeKind::Pli, now + Duration::from_millis(50)));
    }
}
