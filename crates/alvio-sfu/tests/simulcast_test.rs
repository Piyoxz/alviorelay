use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
use alvio_sfu::{BweController, LayerSelector, RtpRouter, StreamConsumer, StreamSource};
use alvio_webrtc::{AlvioRtpPacket, RtpHeader};
use bytes::Bytes;
use std::sync::Arc;

fn make_packet(ssrc: u32, seq: u16, payload: &[u8]) -> AlvioRtpPacket {
    AlvioRtpPacket {
        header: RtpHeader {
            version: 2,
            has_padding: false,
            has_extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 96,
            sequence_number: seq,
            timestamp: 5000 + (seq as u32) * 3000,
            ssrc,
        },
        payload: Bytes::copy_from_slice(payload),
    }
}

#[test]
fn test_simulcast_multi_layer_routing_and_switching() {
    let router = RtpRouter::new();

    let primary_ssrc = 1000;
    let ssrc_low = 1001;
    let ssrc_med = 1002;
    let ssrc_high = 1003;

    // 1. Publisher Alice registers multi-layer Simulcast video track
    let mut alice_source = StreamSource::new(
        TrackId::from("alice-cam-simulcast"),
        PeerId::from("alice"),
        primary_ssrc,
        StreamKind::Video,
        90000,
    );
    alice_source.add_layer(StreamLayer::Low, ssrc_low);
    alice_source.add_layer(StreamLayer::Medium, ssrc_med);
    alice_source.add_layer(StreamLayer::High, ssrc_high);

    router.register_source(Arc::new(alice_source));

    // 2. Subscriber Bob subscribes to the track, initially targeting High layer
    let bob_ssrc = 7777;
    let bob_sub = Arc::new(StreamConsumer::new(
        "bob-video-consumer".to_string(),
        PeerId::from("bob"),
        bob_ssrc,
        100, // Bob initial sequence number
        StreamLayer::High,
    ));
    router.add_consumer(primary_ssrc, Arc::clone(&bob_sub));

    assert_eq!(bob_sub.current_layer(), StreamLayer::High);

    // 3. Alice sends packets on High, Medium, and Low
    let p_high_1 = make_packet(ssrc_high, 10, b"high-frame-1");
    let p_med_1 = make_packet(ssrc_med, 10, b"med-frame-1");
    let p_low_1 = make_packet(ssrc_low, 10, b"low-frame-1");

    let fwd_high = router.route_packet(&p_high_1);
    let fwd_med = router.route_packet(&p_med_1);
    let fwd_low = router.route_packet(&p_low_1);

    // Bob only receives High! Med and Low are gated out
    assert_eq!(fwd_high.len(), 1);
    assert_eq!(fwd_high[0].header.ssrc, bob_ssrc);
    assert_eq!(fwd_high[0].header.sequence_number, 100);
    assert_eq!(fwd_high[0].payload.as_ref(), b"high-frame-1");

    assert!(fwd_med.is_empty(), "Medium packet must be gated out for High subscriber");
    assert!(fwd_low.is_empty(), "Low packet must be gated out for High subscriber");

    // Alice sends second High packet
    let p_high_2 = make_packet(ssrc_high, 11, b"high-frame-2");
    let fwd_high_2 = router.route_packet(&p_high_2);
    assert_eq!(fwd_high_2.len(), 1);
    assert_eq!(fwd_high_2[0].header.sequence_number, 101);

    // 4. Bob requests layer switch to Low (e.g. user minimized window / network bandwidth dropped)
    let switch_ok = router.switch_consumer_layer(primary_ssrc, "bob-video-consumer", StreamLayer::Low);
    assert!(switch_ok);
    assert_eq!(bob_sub.target_layer(), StreamLayer::Low);
    assert_eq!(bob_sub.current_layer(), StreamLayer::High); // Still High until Low keyframe!

    // Alice sends regular (non-keyframe) Low packet
    let p_low_delta = make_packet(ssrc_low, 11, b"low-delta-p-frame");
    let fwd_low_delta = router.route_packet_with_keyframe(&p_low_delta, false);
    assert!(fwd_low_delta.is_empty(), "Delta frame must not trigger premature layer switch");

    // Alice sends High packet in the meantime: Bob still receives it safely
    let p_high_3 = make_packet(ssrc_high, 12, b"high-frame-3");
    let fwd_high_3 = router.route_packet(&p_high_3);
    assert_eq!(fwd_high_3.len(), 1);
    assert_eq!(fwd_high_3[0].header.sequence_number, 102);

    // Now Alice sends a Low Keyframe (I-frame / IDR)!
    let p_low_keyframe = make_packet(ssrc_low, 12, b"low-keyframe-i-frame");
    let fwd_low_key = router.route_packet_with_keyframe(&p_low_keyframe, true);
    assert_eq!(fwd_low_key.len(), 1, "Keyframe must activate the pending layer switch");
    assert_eq!(fwd_low_key[0].header.sequence_number, 103);
    assert_eq!(fwd_low_key[0].payload.as_ref(), b"low-keyframe-i-frame");

    // Now layer switch is completed!
    assert_eq!(bob_sub.current_layer(), StreamLayer::Low);

    // Subsequent High packets are now gated out
    let p_high_4 = make_packet(ssrc_high, 13, b"high-frame-4");
    assert!(router.route_packet(&p_high_4).is_empty());

    // Subsequent Low packets are forwarded with continuous sequence numbering
    let p_low_3 = make_packet(ssrc_low, 13, b"low-frame-3");
    let fwd_low_3 = router.route_packet(&p_low_3);
    assert_eq!(fwd_low_3.len(), 1);
    assert_eq!(fwd_low_3[0].header.sequence_number, 104);
}

#[test]
fn test_adaptive_bwe_layer_selection_integration() {
    let selector = LayerSelector::default();
    let bwe = BweController::new(2_000_000); // 2 Mbps initial

    // Optimal bandwidth -> High
    let layer = selector.select_layer(bwe.current_state().available_bitrate_bps);
    assert_eq!(layer, StreamLayer::High);

    // Network degradation reported from TWCC -> 600 kbps -> Medium
    bwe.update_estimate(600_000, 0.03, 80);
    let layer = selector.select_layer(bwe.current_state().available_bitrate_bps);
    assert_eq!(layer, StreamLayer::Medium);

    // Heavy congestion -> 180 kbps -> Low
    bwe.update_estimate(180_000, 0.12, 250);
    let layer = selector.select_layer(bwe.current_state().available_bitrate_bps);
    assert_eq!(layer, StreamLayer::Low);
}
