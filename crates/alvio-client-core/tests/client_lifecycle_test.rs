use alvio_client_core::ffi::{
    alvio_client_create, alvio_client_destroy, alvio_client_poll_event, alvio_client_state,
};
use alvio_client_core::{ClientEvent, ClientRoom, ConnectionState, MockSignalingTransport};
use alvio_core::{PeerId, RoomId, StreamKind, StreamLayer, TrackId};
use alvio_protocol::{PeerInfo, SignalEnvelope, SignalMessage, TrackInfo};
use std::ffi::CString;

#[tokio::test]
async fn test_client_connect_and_room_lifecycle() {
    let room = ClientRoom::new("test-client-1.0.0");
    assert_eq!(room.state(), ConnectionState::Disconnected);

    let (transport, mut server) = MockSignalingTransport::create_pair(32);
    let mut event_rx = room.subscribe_events();

    // 1. Connect
    room.connect(transport, Some("jwt_secret_token".into()))
        .await
        .expect("Connect should succeed");

    // Expect StateChanged(Connecting)
    let event = event_rx.recv().await.unwrap();
    match event {
        ClientEvent::StateChanged(ConnectionState::Connecting) => {}
        other => panic!("Expected StateChanged(Connecting), got {:?}", other),
    }

    // Verify outbound Connect envelope
    let initial_msg = server
        .recv_from_client()
        .await
        .expect("Server should receive initial envelope");
    match initial_msg.body {
        SignalMessage::Connect {
            token,
            client_version,
        } => {
            assert_eq!(token.as_deref(), Some("jwt_secret_token"));
            assert_eq!(client_version, "test-client-1.0.0");
        }
        other => panic!("Expected Connect message, got {:?}", other),
    }

    // 2. Server sends Ack
    let peer_id = PeerId::new("peer-alice-123");
    server
        .send_to_client(SignalEnvelope::new(SignalMessage::Ack {
            peer_id: peer_id.clone(),
            node_id: "node-relay-1".into(),
        }))
        .await
        .unwrap();

    // Verify Connected event
    let event = event_rx.recv().await.unwrap();
    match event {
        ClientEvent::StateChanged(ConnectionState::Connected) => {}
        other => panic!("Expected StateChanged(Connected), got {:?}", other),
    }

    let event = event_rx.recv().await.unwrap();
    match event {
        ClientEvent::Connected {
            peer_id: p_id,
            node_id,
        } => {
            assert_eq!(p_id, peer_id);
            assert_eq!(node_id, "node-relay-1");
        }
        other => panic!("Expected Connected event, got {:?}", other),
    }

    assert_eq!(room.state(), ConnectionState::Connected);
    assert_eq!(room.self_peer_id(), Some(peer_id.clone()));

    // 3. Client joins room
    let target_room = RoomId::new("room-engineering");
    room.join(
        target_room.clone(),
        "Alice",
        Some(r#"{"role":"presenter"}"#.into()),
    )
    .await
    .unwrap();

    let join_msg = server.recv_from_client().await.unwrap();
    match join_msg.body {
        SignalMessage::Join {
            room_id,
            peer_name,
            metadata,
        } => {
            assert_eq!(room_id, target_room);
            assert_eq!(peer_name, "Alice");
            assert_eq!(metadata.as_deref(), Some(r#"{"role":"presenter"}"#));
        }
        other => panic!("Expected Join, got {:?}", other),
    }

    // Server sends RoomJoined with snapshot of existing peer Bob
    let bob_peer = PeerInfo {
        id: PeerId::new("peer-bob-456"),
        name: "Bob".into(),
        metadata: None,
    };
    let bob_track = TrackInfo {
        id: TrackId::new("trk-bob-audio"),
        peer_id: bob_peer.id.clone(),
        kind: StreamKind::Audio,
        source: "microphone".into(),
        layers: vec![StreamLayer::High],
    };

    server
        .send_to_client(SignalEnvelope::new(SignalMessage::RoomJoined {
            room_id: target_room.clone(),
            self_peer_id: peer_id.clone(),
            peers: vec![bob_peer.clone()],
            active_tracks: vec![bob_track.clone()],
        }))
        .await
        .unwrap();

    // Verify RoomJoined state
    loop {
        if room.state() == ConnectionState::InRoom {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    assert_eq!(room.room_id(), Some(target_room));
    assert_eq!(room.peers().len(), 1);
    assert_eq!(room.peers()[0].name, "Bob");
    assert_eq!(room.tracks().len(), 1);
    assert_eq!(room.tracks()[0].id.as_str(), "trk-bob-audio");

    // 4. Remote peer publishes a new camera track
    let bob_video_track = TrackInfo {
        id: TrackId::new("trk-bob-video"),
        peer_id: bob_peer.id.clone(),
        kind: StreamKind::Video,
        source: "camera".into(),
        layers: vec![StreamLayer::Low, StreamLayer::Medium, StreamLayer::High],
    };
    server
        .send_to_client(SignalEnvelope::new(SignalMessage::TrackPublished {
            track: bob_video_track.clone(),
        }))
        .await
        .unwrap();

    // Check track added to client local state
    tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    assert_eq!(room.tracks().len(), 2);

    // 5. Data message exchange
    server
        .send_to_client(SignalEnvelope::new(SignalMessage::DataReceived {
            source_peer_id: bob_peer.id.clone(),
            payload: "Hello Alice!".into(),
        }))
        .await
        .unwrap();

    let mut found_data = false;
    for _ in 0..20 {
        while let Ok(evt) = event_rx.try_recv() {
            if let ClientEvent::DataReceived {
                source_peer_id,
                payload,
            } = evt
            {
                assert_eq!(source_peer_id, bob_peer.id.clone());
                assert_eq!(payload, "Hello Alice!");
                found_data = true;
                break;
            }
        }
        if found_data {
            break;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
    assert!(found_data);

    // 6. Leave room
    room.leave().await.unwrap();
    assert_eq!(room.state(), ConnectionState::Connected);
    assert_eq!(room.room_id(), None);
    assert_eq!(room.peers().len(), 0);
    assert_eq!(room.tracks().len(), 0);

    room.disconnect().await;
    assert_eq!(room.state(), ConnectionState::Disconnected);
}

#[test]
fn test_ffi_client_lifecycle() {
    let version_c = CString::new("native-c-1.0.0").unwrap();
    let handle = alvio_client_create(version_c.as_ptr());
    assert!(!handle.is_null());

    let state = alvio_client_state(handle);
    assert_eq!(state, 0); // Disconnected

    // Initially no events queued
    let evt = alvio_client_poll_event(handle);
    assert!(evt.is_null());

    // Clean up
    alvio_client_destroy(handle);
}
