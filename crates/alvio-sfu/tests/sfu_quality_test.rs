use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
use alvio_sfu::{
    BweController, KeyframeKind, NackGenerator, RtpRouter, StreamConsumer, StreamSource,
};
use alvio_webrtc::{AlvioRtpPacket, RtpHeader};
use bytes::Bytes;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn create_rtp_packet(ssrc: u32, seq: u16, payload_str: &str) -> AlvioRtpPacket {
    AlvioRtpPacket {
        header: RtpHeader {
            version: 2,
            has_padding: false,
            has_extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 96,
            sequence_number: seq,
            timestamp: 10000 + (seq as u32) * 3000,
            ssrc,
        },
        payload: Bytes::copy_from_slice(payload_str.as_bytes()),
    }
}

#[test]
fn test_sfu_packet_loss_recovery_with_nack_buffer() {
    let router = RtpRouter::new();

    let pub_ssrc = 1000;
    let sub_ssrc = 2000;

    let source = Arc::new(StreamSource::new(
        TrackId::from("camera-alice"),
        PeerId::from("alice"),
        pub_ssrc,
        StreamKind::Video,
        90000,
    ));
    router.register_source(source);

    let consumer = Arc::new(StreamConsumer::new(
        "sub-bob".to_string(),
        PeerId::from("bob"),
        sub_ssrc,
        0,
        StreamLayer::High,
    ));
    router.add_consumer(pub_ssrc, consumer);

    // 1. Publisher sends 50 packets (seq 1 to 50)
    for seq in 1..=50 {
        let packet = create_rtp_packet(pub_ssrc, seq, &format!("frame-payload-{seq}"));
        let forwarded = router.route_packet(&packet);
        assert_eq!(forwarded.len(), 1);
        assert_eq!(forwarded[0].header.ssrc, sub_ssrc);
    }

    // 2. Simulate Bob receiving packets with loss of seq 24 and 25
    let mut bob_loss_detector = NackGenerator::new(3, Duration::from_millis(50));
    let now = Instant::now();

    // Bob receives 1..=23
    for seq in 1..=23 {
        bob_loss_detector.on_packet(seq, now);
    }
    assert_eq!(bob_loss_detector.missing_count(), 0);

    // Gap: seq 24 and 25 lost, seq 26 arrives
    bob_loss_detector.on_packet(26, now);
    assert_eq!(bob_loss_detector.missing_count(), 2);

    // Bob detects missing packets and generates NACK
    let nack_requests = bob_loss_detector.generate_nacks(now);
    assert_eq!(nack_requests, vec![24, 25]);

    // 3. SFU receives NACK and recovers lost packets from its internal NackBuffer
    let recovered = router.handle_nack(pub_ssrc, &nack_requests);
    assert_eq!(recovered.len(), 2);
    assert_eq!(recovered[0].header.sequence_number, 24);
    assert_eq!(recovered[0].payload.as_ref(), b"frame-payload-24");
    assert_eq!(recovered[1].header.sequence_number, 25);
    assert_eq!(recovered[1].payload.as_ref(), b"frame-payload-25");

    // 4. Bob receives retransmitted packets
    bob_loss_detector.on_packet(24, now);
    bob_loss_detector.on_packet(25, now);
    assert_eq!(bob_loss_detector.missing_count(), 0);
}

#[test]
fn test_sfu_keyframe_storm_protection() {
    let router = RtpRouter::new();
    let source_ssrc = 7777;

    let source = Arc::new(StreamSource::new(
        TrackId::from("screen-alice"),
        PeerId::from("alice"),
        source_ssrc,
        StreamKind::Video,
        90000,
    ));
    router.register_source(source);

    let start = Instant::now();

    // 5 subscribers join almost at the same time and request PLI
    let p1 = router.request_keyframe(source_ssrc, KeyframeKind::Pli, start);
    let p2 = router.request_keyframe(
        source_ssrc,
        KeyframeKind::Pli,
        start + Duration::from_millis(10),
    );
    let p3 = router.request_keyframe(
        source_ssrc,
        KeyframeKind::Pli,
        start + Duration::from_millis(50),
    );
    let p4 = router.request_keyframe(
        source_ssrc,
        KeyframeKind::Pli,
        start + Duration::from_millis(100),
    );
    let p5 = router.request_keyframe(
        source_ssrc,
        KeyframeKind::Pli,
        start + Duration::from_millis(200),
    );

    // Only the first one is dispatched upstream; the other 4 are coalesced/throttled!
    assert!(p1, "First PLI must be forwarded to publisher");
    assert!(!p2, "Subsequent rapid PLIs must be suppressed");
    assert!(!p3, "Subsequent rapid PLIs must be suppressed");
    assert!(!p4, "Subsequent rapid PLIs must be suppressed");
    assert!(!p5, "Subsequent rapid PLIs must be suppressed");

    // After default cooldown (500ms), next PLI is allowed through
    let p6 = router.request_keyframe(
        source_ssrc,
        KeyframeKind::Pli,
        start + Duration::from_millis(550),
    );
    assert!(p6, "PLI after cooldown must be forwarded to publisher");
}

#[test]
fn test_sfu_bwe_congestion_control_feedback() {
    let bwe = BweController::new(2_500_000); // 2.5 Mbps
    let (low_threshold, mid_threshold) = (400_000, 1_200_000);

    // Initial state: High layer recommended
    assert_eq!(
        bwe.recommend_layer(low_threshold, mid_threshold),
        StreamLayer::High
    );

    // Network congestion: bitrate drops to 900 kbps, 5% loss, 120ms RTT
    bwe.update_estimate(900_000, 0.05, 120);
    assert_eq!(
        bwe.recommend_layer(low_threshold, mid_threshold),
        StreamLayer::Medium
    );

    // Severe congestion: bitrate drops to 250 kbps, 18% loss, 350ms RTT
    bwe.update_estimate(250_000, 0.18, 350);
    assert_eq!(
        bwe.recommend_layer(low_threshold, mid_threshold),
        StreamLayer::Low
    );

    // Network recovery: bitrate back to 3 Mbps
    bwe.update_estimate(3_000_000, 0.0, 30);
    assert_eq!(
        bwe.recommend_layer(low_threshold, mid_threshold),
        StreamLayer::High
    );
}
