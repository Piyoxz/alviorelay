use alvio_core::{PeerId, RoomId, StreamKind, StreamLayer, TrackId};
use alvio_egress::{OutputFormat, RecordingConfig, RecordingManager};
use alvio_sfu::{RtpRouter, StreamConsumer, StreamSource};
use alvio_storage::{LocalStorage, StorageBackend};
use alvio_webrtc::{AlvioRtpPacket, RtpHeader};
use bytes::Bytes;
use std::sync::Arc;

fn make_packet(ssrc: u32, seq: u16, payload_str: &str) -> AlvioRtpPacket {
    AlvioRtpPacket {
        header: RtpHeader {
            version: 2,
            has_padding: false,
            has_extension: false,
            csrc_count: 0,
            marker: false,
            payload_type: 96,
            sequence_number: seq,
            timestamp: 1000 + (seq as u32) * 160,
            ssrc,
        },
        payload: Bytes::copy_from_slice(payload_str.as_bytes()),
    }
}

#[tokio::test]
async fn test_full_egress_recording_pipeline_with_storage() {
    let tmp_dir = std::env::temp_dir().join(format!(
        "alvio_egress_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let storage: Arc<dyn StorageBackend> = Arc::new(LocalStorage::new(&tmp_dir));
    let manager = RecordingManager::new(Arc::clone(&storage));

    let room_id = RoomId::from("room-presentation-101");
    let dest_filename = "presentation_recording.mp4";

    let config = RecordingConfig {
        room_id: room_id.clone(),
        format: OutputFormat::Mp4,
        destination_path: dest_filename.to_string(),
        record_audio: true,
        record_video: true,
    };

    // 1. Start Recording session
    let (session_id, tap) = manager.start_recording(config).unwrap();
    assert!(manager.is_room_recording(&room_id));

    // 2. Setup SFU Router with Alice's video source and Bob's consumer
    let router = RtpRouter::new();
    let alice_ssrc = 1111;
    let alice_source = Arc::new(StreamSource::new(
        TrackId::from("alice-video"),
        PeerId::from("alice"),
        alice_ssrc,
        StreamKind::Video,
        90000,
    ));
    router.register_source(alice_source);

    let bob_sub = Arc::new(StreamConsumer::new(
        "bob-sub".to_string(),
        PeerId::from("bob"),
        9999,
        0,
        StreamLayer::High,
    ));
    router.add_consumer(alice_ssrc, bob_sub);

    // 3. Simulate streaming 25 video packets
    let mut total_bytes = 0;
    for seq in 1..=25 {
        let payload = format!("video-frame-data-{seq}");
        total_bytes += payload.len() as u64;
        let pkt = make_packet(alice_ssrc, seq, &payload);

        // Forward to subscribers in room
        let forwarded = router.route_packet(&pkt);
        assert_eq!(forwarded.len(), 1);

        // Tap simultaneously captures out-of-band
        tap.push_packet(pkt);
    }

    assert_eq!(tap.packets_captured(), 25);
    assert_eq!(tap.bytes_captured(), total_bytes);

    // 4. Stop Recording session
    let output = manager.stop_recording(&session_id).unwrap();
    assert_eq!(output.packets_captured, 25);
    assert_eq!(output.bytes_captured, total_bytes);
    assert_eq!(output.format, OutputFormat::Mp4);
    assert!(!manager.is_room_recording(&room_id));

    // 5. Store the recorded asset into StorageBackend
    let dummy_video_file = Bytes::from_static(b"RIFF-simulated-encoded-mp4-video-container");
    let stored_meta = storage.put(&output.destination_path, dummy_video_file.clone()).await.unwrap();
    assert_eq!(stored_meta.path, dest_filename);
    assert_eq!(stored_meta.content_type, "video/mp4");
    assert!(storage.exists(dest_filename).await.unwrap());

    let fetched = storage.get(dest_filename).await.unwrap();
    assert_eq!(fetched, dummy_video_file);

    // Clean up
    storage.delete(dest_filename).await.unwrap();
    assert!(!storage.exists(dest_filename).await.unwrap());
}
