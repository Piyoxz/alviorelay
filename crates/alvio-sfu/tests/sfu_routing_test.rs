use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
use alvio_sfu::{RtpRouter, StreamConsumer, StreamSource};
use alvio_webrtc::AlvioRtpPacket;
use bytes::Bytes;
use std::sync::Arc;

#[test]
fn test_sfu_multi_track_routing() {
    let router = RtpRouter::new();

    // Source 1: Alice Audio
    let audio_source = Arc::new(StreamSource::new(
        TrackId::from("alice-audio"),
        PeerId::from("alice"),
        1111,
        StreamKind::Audio,
        48000,
    ));
    router.register_source(audio_source);

    // Source 2: Alice Video
    let video_source = Arc::new(StreamSource::new(
        TrackId::from("alice-video"),
        PeerId::from("alice"),
        2222,
        StreamKind::Video,
        90000,
    ));
    router.register_source(video_source);

    // Subscriber Bob subscribes to Alice's Video
    let bob_video_sub = Arc::new(StreamConsumer::new(
        "sub-bob-video".to_string(),
        PeerId::from("bob"),
        9999, // Bob's negotiated video SSRC
        0,
        StreamLayer::High,
    ));
    router.add_consumer(2222, bob_video_sub);

    // Packet arrives on Audio (1111) -> Bob shouldn't receive it because he only subscribed to video
    let audio_raw = vec![
        0x80, 0x6F, 0x00, 0x01,
        0x00, 0x00, 0x03, 0xE8,
        0x00, 0x00, 0x04, 0x57, // SSRC 1111
        0x12, 0x34,
    ];
    let audio_packet = AlvioRtpPacket::parse(Bytes::from(audio_raw)).unwrap();
    let routed_audio = router.route_packet(&audio_packet);
    assert!(routed_audio.is_empty(), "Audio packet with no subscribers should not be routed");

    // Packet arrives on Video (2222) -> Bob receives it
    let video_raw = vec![
        0x80, 0x60, 0x00, 0x01,
        0x00, 0x01, 0x5F, 0x90,
        0x00, 0x00, 0x08, 0xAE, // SSRC 2222
        0xDE, 0xAD, 0xBE, 0xEF,
    ];
    let video_packet = AlvioRtpPacket::parse(Bytes::from(video_raw)).unwrap();
    let routed_video = router.route_packet(&video_packet);
    assert_eq!(routed_video.len(), 1);
    assert_eq!(routed_video[0].header.ssrc, 9999);
    assert_eq!(routed_video[0].header.sequence_number, 0);
    assert_eq!(routed_video[0].payload.as_ref(), &[0xDE, 0xAD, 0xBE, 0xEF]);
}
