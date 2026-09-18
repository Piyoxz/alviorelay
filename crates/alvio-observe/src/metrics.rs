use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

static GLOBAL_METRICS: OnceLock<AlvioMetrics> = OnceLock::new();

/// Global accessor to the shared AlvioRelay metrics registry.
pub fn get_metrics() -> &'static AlvioMetrics {
    GLOBAL_METRICS.get_or_init(AlvioMetrics::new)
}

/// Production Prometheus & OpenMetrics registry for AlvioRelay.
///
/// Implemented entirely using lock-free atomics to ensure updating counters on the
/// hot RTP routing loop takes sub-2ns with zero lock contention.
pub struct AlvioMetrics {
    // Media Hot Path Counters
    pub packets_in: AtomicU64,
    pub packets_out: AtomicU64,
    pub packets_dropped: AtomicU64,
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,

    // Feedback & Recovery Counters
    pub nack_requests: AtomicU64,
    pub pli_requests: AtomicU64,

    // Non-Media & Egress Counters
    pub data_packets: AtomicU64,
    pub webhooks_dispatched: AtomicU64,
    pub webhooks_delivered: AtomicU64,

    // Active State Gauges
    pub rooms_active: AtomicU64,
    pub peers_active: AtomicU64,
    pub tracks_active: AtomicU64,
    pub recordings_active: AtomicU64,
    pub whip_sessions_active: AtomicU64,
}

impl Default for AlvioMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl AlvioMetrics {
    pub fn new() -> Self {
        Self {
            packets_in: AtomicU64::new(0),
            packets_out: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            nack_requests: AtomicU64::new(0),
            pli_requests: AtomicU64::new(0),
            data_packets: AtomicU64::new(0),
            webhooks_dispatched: AtomicU64::new(0),
            webhooks_delivered: AtomicU64::new(0),
            rooms_active: AtomicU64::new(0),
            peers_active: AtomicU64::new(0),
            tracks_active: AtomicU64::new(0),
            recordings_active: AtomicU64::new(0),
            whip_sessions_active: AtomicU64::new(0),
        }
    }

    /// Increments incoming packet count and byte volume atomically.
    pub fn record_packet_in(&self, bytes: u64) {
        self.packets_in.fetch_add(1, Ordering::Relaxed);
        self.bytes_in.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increments outgoing forwarded packet count and byte volume atomically.
    pub fn record_packet_out(&self, bytes: u64) {
        self.packets_out.fetch_add(1, Ordering::Relaxed);
        self.bytes_out.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Formats all metrics into standard Prometheus OpenMetrics exposition text format.
    pub fn render_prometheus(&self) -> String {
        let mut out = String::with_capacity(2048);

        macro_rules! emit_counter {
            ($name:literal, $help:literal, $field:ident) => {
                out.push_str(concat!("# HELP ", $name, " ", $help, "\n"));
                out.push_str(concat!("# TYPE ", $name, " counter\n"));
                out.push_str(&format!(
                    "{} {}\n\n",
                    $name,
                    self.$field.load(Ordering::Relaxed)
                ));
            };
        }

        macro_rules! emit_gauge {
            ($name:literal, $help:literal, $field:ident) => {
                out.push_str(concat!("# HELP ", $name, " ", $help, "\n"));
                out.push_str(concat!("# TYPE ", $name, " gauge\n"));
                out.push_str(&format!(
                    "{} {}\n\n",
                    $name,
                    self.$field.load(Ordering::Relaxed)
                ));
            };
        }

        // Emit Hot Path Counters
        emit_counter!(
            "alvio_packets_in_total",
            "Total incoming media RTP packets received",
            packets_in
        );
        emit_counter!(
            "alvio_packets_out_total",
            "Total media RTP packets forwarded to subscribers",
            packets_out
        );
        emit_counter!(
            "alvio_packets_dropped_total",
            "Total media packets dropped due to buffer limits or congestion",
            packets_dropped
        );
        emit_counter!(
            "alvio_bytes_in_total",
            "Total incoming media bytes received",
            bytes_in
        );
        emit_counter!(
            "alvio_bytes_out_total",
            "Total outgoing media bytes forwarded",
            bytes_out
        );

        // Emit Quality & Feedback Counters
        emit_counter!(
            "alvio_nack_requests_total",
            "Total NACK packet retransmission requests handled",
            nack_requests
        );
        emit_counter!(
            "alvio_pli_requests_total",
            "Total Picture Loss Indication (PLI) keyframe requests handled",
            pli_requests
        );

        // Emit Non-Media & Egress Counters
        emit_counter!(
            "alvio_data_packets_total",
            "Total WebRTC DataChannel packets routed",
            data_packets
        );
        emit_counter!(
            "alvio_webhooks_dispatched_total",
            "Total outbound webhook notifications dispatched",
            webhooks_dispatched
        );
        emit_counter!(
            "alvio_webhooks_delivered_total",
            "Total outbound webhook notifications successfully delivered",
            webhooks_delivered
        );

        // Emit Active Gauges
        emit_gauge!(
            "alvio_rooms_active",
            "Number of active media rooms currently in memory",
            rooms_active
        );
        emit_gauge!(
            "alvio_peers_active",
            "Number of active connected peers across all rooms",
            peers_active
        );
        emit_gauge!(
            "alvio_tracks_active",
            "Number of active media tracks currently published",
            tracks_active
        );
        emit_gauge!(
            "alvio_recordings_active",
            "Number of egress recording sessions currently active",
            recordings_active
        );
        emit_gauge!(
            "alvio_whip_sessions_active",
            "Number of WHIP broadcast sessions currently publishing",
            whip_sessions_active
        );

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_metric_counters_and_gauges() {
        let metrics = AlvioMetrics::new();
        metrics.record_packet_in(1200);
        metrics.record_packet_out(1200);
        metrics.packets_dropped.fetch_add(1, Ordering::Relaxed);
        metrics.rooms_active.store(5, Ordering::Relaxed);
        metrics.peers_active.store(15, Ordering::Relaxed);

        assert_eq!(metrics.packets_in.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.bytes_in.load(Ordering::Relaxed), 1200);
        assert_eq!(metrics.packets_out.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.packets_dropped.load(Ordering::Relaxed), 1);
        assert_eq!(metrics.rooms_active.load(Ordering::Relaxed), 5);
        assert_eq!(metrics.peers_active.load(Ordering::Relaxed), 15);

        let rendered = metrics.render_prometheus();
        assert!(rendered.contains("# HELP alvio_packets_in_total"));
        assert!(rendered.contains("# TYPE alvio_packets_in_total counter"));
        assert!(rendered.contains("alvio_packets_in_total 1"));
        assert!(rendered.contains("alvio_rooms_active 5"));
        assert!(rendered.contains("alvio_peers_active 15"));
    }
}
