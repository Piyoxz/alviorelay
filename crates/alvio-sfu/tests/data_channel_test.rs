use alvio_core::PeerId;
use alvio_sfu::{DataChannelMetadata, DataMessage, DataRouter};

#[test]
fn test_data_channel_room_routing_lifecycle() {
    let router = DataRouter::new();

    let alice = PeerId::from("alice-user");
    let bob = PeerId::from("bob-user");
    let charlie = PeerId::from("charlie-user");

    router.register_peer(alice.clone());
    router.register_peer(bob.clone());
    router.register_peer(charlie.clone());

    assert_eq!(router.active_peer_count(), 3);

    // Declare channels
    router.declare_channel(DataChannelMetadata::reliable("chat"));
    router.declare_channel(DataChannelMetadata::lossy("cursor", 0));
    router.declare_channel(DataChannelMetadata::reliable("whiteboard"));

    // Subscriptions
    router.subscribe(&alice, "chat");
    router.subscribe(&alice, "whiteboard");

    router.subscribe(&bob, "chat");
    router.subscribe(&bob, "cursor");

    router.subscribe(&charlie, "chat");
    router.subscribe(&charlie, "cursor");
    router.subscribe(&charlie, "whiteboard");

    // 1. Broadcast Chat (Reliable Text) from Alice
    let chat_msg = DataMessage::text(alice.clone(), "chat", "Welcome to AlvioRelay Room!");
    let chat_recipients = router.route_broadcast(&chat_msg);

    // Both Bob and Charlie should receive it; Alice is the sender and excluded
    assert_eq!(chat_recipients.len(), 2);
    assert!(chat_recipients.contains(&bob));
    assert!(chat_recipients.contains(&charlie));
    assert!(!chat_recipients.contains(&alice));

    // 2. Broadcast Cursor (Unreliable Binary) from Bob
    let cursor_payload = vec![0x00, 0x64, 0x00, 0xC8]; // x=100, y=200
    let cursor_msg = DataMessage::binary(bob.clone(), "cursor", cursor_payload);
    let cursor_recipients = router.route_broadcast(&cursor_msg);

    // Only Charlie is subscribed to cursor (Alice is NOT subscribed to cursor)
    assert_eq!(cursor_recipients.len(), 1);
    assert_eq!(cursor_recipients[0], charlie);

    // 3. Direct / Unicast Messaging (Alice to Charlie on "whiteboard")
    let wb_msg = DataMessage::text(alice.clone(), "whiteboard", "Drawing rect(10, 20, 100, 50)");
    assert!(router.route_direct(&wb_msg, &charlie));
    // Bob did not subscribe to "whiteboard"
    assert!(!router.route_direct(&wb_msg, &bob));

    // 4. Unregister Peer (Bob leaves room)
    router.unregister_peer(&bob);
    assert_eq!(router.active_peer_count(), 2);

    let chat_msg_2 = DataMessage::text(alice.clone(), "chat", "Bob has left.");
    let chat_recipients_2 = router.route_broadcast(&chat_msg_2);
    assert_eq!(chat_recipients_2, vec![charlie.clone()]);
}
