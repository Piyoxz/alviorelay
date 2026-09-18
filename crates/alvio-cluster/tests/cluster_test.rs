use alvio_cluster::{
    ClusterNode, ClusterRegistry, DrainController, NodeStatus,
};
use alvio_core::RoomId;

#[test]
fn test_multi_node_authoritative_routing() {
    let registry = ClusterRegistry::default();

    let node_us = ClusterNode::new("edge-us-1", "http://us1.alvio.io:7880", "us1.alvio.io:7882", "us-east", 500);
    let node_eu = ClusterNode::new("edge-eu-1", "http://eu1.alvio.io:7880", "eu1.alvio.io:7882", "eu-west", 500);
    let node_ap = ClusterNode::new("edge-ap-1", "http://ap1.alvio.io:7880", "ap1.alvio.io:7882", "ap-southeast", 500);

    registry.register(node_us).unwrap();
    registry.register(node_eu).unwrap();
    registry.register(node_ap).unwrap();

    assert_eq!(registry.node_count(), 3);

    // Verify 20 rooms map deterministically
    for i in 0..20 {
        let room = RoomId::from(format!("meeting-conf-{i}"));
        let pick_first = registry.locate_authoritative_node(&room).unwrap();
        let pick_second = registry.locate_authoritative_node(&room).unwrap();

        assert_eq!(
            pick_first.id, pick_second.id,
            "Room routing must be 100% deterministic for identical room IDs"
        );
        assert!(pick_first.is_available());
    }
}

#[test]
fn test_node_drain_and_failover() {
    let registry = ClusterRegistry::default();

    let node_a = ClusterNode::new("node-alpha", "http://alpha:7880", "alpha:7882", "us-east", 100);
    let node_b = ClusterNode::new("node-bravo", "http://bravo:7880", "bravo:7882", "us-east", 100);

    registry.register(node_a).unwrap();
    registry.register(node_b).unwrap();

    let room = RoomId::from("critical-stream-room");
    let initial_node = registry.locate_authoritative_node(&room).unwrap();
    let initial_id = initial_node.id.clone();

    // Node enters Drain mode (e.g. rolling maintenance)
    initial_node.set_status(NodeStatus::Draining);
    assert!(!initial_node.is_available());

    // Next room lookup must gracefully shift to the remaining active peer node
    let failover_node = registry.locate_authoritative_node(&room).unwrap();
    assert_ne!(failover_node.id, initial_id);
    assert_eq!(failover_node.status(), NodeStatus::Active);

    // If both nodes drain, lookup returns NoHealthyNodes error
    failover_node.set_status(NodeStatus::Draining);
    let err = registry.locate_authoritative_node(&room);
    assert!(err.is_err());
}

#[test]
fn test_drain_controller_toggling() {
    let drain = DrainController::new();
    assert!(!drain.is_draining());
    assert!(drain.can_accept_new_room());

    let transitioned = drain.start_drain();
    assert!(transitioned);
    assert!(drain.is_draining());
    assert!(!drain.can_accept_new_room());

    let second_transition = drain.start_drain();
    assert!(!second_transition, "Should return false if already in drain state");
}
