use alvio_core::{RoomId, StreamKind, StreamLayer};
use alvio_protocol::{SignalEnvelope, SignalMessage};
use alvio_signal::{create_signaling_router, RoomRegistry};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::test]
async fn test_full_signaling_lifecycle() {
    let registry = RoomRegistry::new(10);
    let app = create_signaling_router(registry, "test-node".to_string());

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_url = format!("ws://{addr}/ws");

    // Connect Client 1 (Alice)
    let (ws_stream1, _) = connect_async(&ws_url).await.expect("Client 1 should connect");
    let (mut write1, mut read1) = ws_stream1.split();

    // Client 1 should receive ACK
    let first_msg = read1.next().await.unwrap().unwrap();
    let ack_env = SignalEnvelope::from_json(first_msg.to_text().unwrap()).unwrap();
    let alice_peer_id = match ack_env.body {
        SignalMessage::Ack { peer_id, node_id } => {
            assert_eq!(node_id, "test-node");
            peer_id
        }
        other => panic!("Expected Ack, got {other:?}"),
    };

    // Client 1 joins room
    let join_msg = SignalEnvelope::new(SignalMessage::Join {
        room_id: RoomId::from("room-alpha"),
        peer_name: "Alice".to_string(),
        metadata: None,
    });
    write1
        .send(Message::Text(join_msg.to_json().unwrap().into()))
        .await
        .unwrap();

    // Client 1 receives RoomJoined
    let room_joined_msg = read1.next().await.unwrap().unwrap();
    let joined_env = SignalEnvelope::from_json(room_joined_msg.to_text().unwrap()).unwrap();
    match joined_env.body {
        SignalMessage::RoomJoined {
            room_id,
            self_peer_id,
            peers,
            active_tracks,
        } => {
            assert_eq!(room_id.as_str(), "room-alpha");
            assert_eq!(self_peer_id, alice_peer_id);
            assert_eq!(peers.len(), 1);
            assert_eq!(active_tracks.len(), 0);
        }
        other => panic!("Expected RoomJoined, got {other:?}"),
    }

    // Connect Client 2 (Bob)
    let (ws_stream2, _) = connect_async(&ws_url).await.expect("Client 2 should connect");
    let (mut write2, mut read2) = ws_stream2.split();

    // Client 2 receives ACK
    let ack2_msg = read2.next().await.unwrap().unwrap();
    let ack2_env = SignalEnvelope::from_json(ack2_msg.to_text().unwrap()).unwrap();
    let bob_peer_id = match ack2_env.body {
        SignalMessage::Ack { peer_id, .. } => peer_id,
        other => panic!("Expected Ack for Bob, got {other:?}"),
    };

    // Client 2 joins same room
    let join2_msg = SignalEnvelope::new(SignalMessage::Join {
        room_id: RoomId::from("room-alpha"),
        peer_name: "Bob".to_string(),
        metadata: None,
    });
    write2
        .send(Message::Text(join2_msg.to_json().unwrap().into()))
        .await
        .unwrap();

    // Client 2 receives RoomJoined with Alice in it
    let bob_joined = read2.next().await.unwrap().unwrap();
    let bob_joined_env = SignalEnvelope::from_json(bob_joined.to_text().unwrap()).unwrap();
    match bob_joined_env.body {
        SignalMessage::RoomJoined { peers, .. } => {
            assert_eq!(peers.len(), 2);
            assert!(peers.iter().any(|p| p.id == alice_peer_id));
            assert!(peers.iter().any(|p| p.id == bob_peer_id));
        }
        other => panic!("Expected RoomJoined for Bob, got {other:?}"),
    }

    // Client 1 receives PeerJoined notification for Bob
    let peer_joined_msg = read1.next().await.unwrap().unwrap();
    let peer_joined_env = SignalEnvelope::from_json(peer_joined_msg.to_text().unwrap()).unwrap();
    match peer_joined_env.body {
        SignalMessage::PeerJoined { peer } => {
            assert_eq!(peer.id, bob_peer_id);
            assert_eq!(peer.name, "Bob");
        }
        other => panic!("Expected PeerJoined on Alice, got {other:?}"),
    }

    // Client 1 publishes a video track
    let pub_track = SignalEnvelope::new(SignalMessage::PublishTrack {
        kind: StreamKind::Video,
        source: "camera".to_string(),
        layers: vec![StreamLayer::Low, StreamLayer::High],
    });
    write1
        .send(Message::Text(pub_track.to_json().unwrap().into()))
        .await
        .unwrap();

    // Client 1 receives TrackPublished confirmation
    let alice_track_pub = read1.next().await.unwrap().unwrap();
    let alice_track_env = SignalEnvelope::from_json(alice_track_pub.to_text().unwrap()).unwrap();
    let track_id = match alice_track_env.body {
        SignalMessage::TrackPublished { track } => {
            assert_eq!(track.peer_id, alice_peer_id);
            assert_eq!(track.kind, StreamKind::Video);
            track.id
        }
        other => panic!("Expected TrackPublished for Alice, got {other:?}"),
    };

    // Client 2 (Bob) receives TrackPublished broadcast
    let bob_track_msg = read2.next().await.unwrap().unwrap();
    let bob_track_env = SignalEnvelope::from_json(bob_track_msg.to_text().unwrap()).unwrap();
    match bob_track_env.body {
        SignalMessage::TrackPublished { track } => {
            assert_eq!(track.id, track_id);
            assert_eq!(track.peer_id, alice_peer_id);
        }
        other => panic!("Expected TrackPublished broadcast on Bob, got {other:?}"),
    }

    // Client 2 leaves
    let leave_msg = SignalEnvelope::new(SignalMessage::Leave);
    write2
        .send(Message::Text(leave_msg.to_json().unwrap().into()))
        .await
        .unwrap();

    // Client 1 receives PeerLeft notification
    let left_msg = read1.next().await.unwrap().unwrap();
    let left_env = SignalEnvelope::from_json(left_msg.to_text().unwrap()).unwrap();
    match left_env.body {
        SignalMessage::PeerLeft { peer_id, reason } => {
            assert_eq!(peer_id, bob_peer_id);
            assert_eq!(reason, "voluntary_leave");
        }
        other => panic!("Expected PeerLeft on Alice, got {other:?}"),
    }
}
