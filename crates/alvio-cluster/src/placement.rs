use crate::error::{ClusterError, ClusterResult};
use crate::node::ClusterNode;
use alvio_core::RoomId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Trait defining room-to-node placement strategies in the cluster.
pub trait PlacementStrategy: Send + Sync {
    fn select_node<'a>(
        &self,
        room_id: &RoomId,
        candidates: &'a [Arc<ClusterNode>],
    ) -> ClusterResult<&'a Arc<ClusterNode>>;
}

/// Rendezvous (Highest Random Weight / HRW) Consistent Hashing placement.
///
/// Ensures all participants connecting to the same `RoomId` route to the same authoritative
/// node deterministically with zero database or cross-node coordination overhead.
#[derive(Default, Clone)]
pub struct ConsistentHashPlacement;

impl ConsistentHashPlacement {
    pub fn new() -> Self {
        Self
    }

    fn calculate_weight(room_id: &str, node_id: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        room_id.hash(&mut hasher);
        node_id.hash(&mut hasher);
        hasher.finish()
    }
}

impl PlacementStrategy for ConsistentHashPlacement {
    fn select_node<'a>(
        &self,
        room_id: &RoomId,
        candidates: &'a [Arc<ClusterNode>],
    ) -> ClusterResult<&'a Arc<ClusterNode>> {
        let available: Vec<&Arc<ClusterNode>> = candidates
            .iter()
            .filter(|n| n.is_available())
            .collect();

        if available.is_empty() {
            return Err(ClusterError::NoHealthyNodes);
        }

        // Find candidate with maximum weight for (room_id, node.id)
        let best_node = available
            .into_iter()
            .max_by_key(|node| Self::calculate_weight(room_id.as_str(), &node.id))
            .ok_or(ClusterError::NoHealthyNodes)?;

        Ok(best_node)
    }
}

/// Least-Loaded placement strategy.
///
/// Selects the healthy node with the lowest active room load ratio.
#[derive(Default, Clone)]
pub struct LeastLoadedPlacement;

impl LeastLoadedPlacement {
    pub fn new() -> Self {
        Self
    }
}

impl PlacementStrategy for LeastLoadedPlacement {
    fn select_node<'a>(
        &self,
        _room_id: &RoomId,
        candidates: &'a [Arc<ClusterNode>],
    ) -> ClusterResult<&'a Arc<ClusterNode>> {
        let available: Vec<&Arc<ClusterNode>> = candidates
            .iter()
            .filter(|n| n.is_available())
            .collect();

        if available.is_empty() {
            return Err(ClusterError::NoHealthyNodes);
        }

        let best_node = available
            .into_iter()
            .min_by(|a, b| {
                a.load_factor()
                    .partial_cmp(&b.load_factor())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or(ClusterError::NoHealthyNodes)?;

        Ok(best_node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::NodeStatus;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_consistent_hash_deterministic_placement() {
        let node1 = Arc::new(ClusterNode::new("node-1", "http://node1:7880", "node1:7882", "us-east", 100));
        let node2 = Arc::new(ClusterNode::new("node-2", "http://node2:7880", "node2:7882", "us-east", 100));
        let node3 = Arc::new(ClusterNode::new("node-3", "http://node3:7880", "node3:7882", "us-east", 100));
        let candidates = vec![node1.clone(), node2.clone(), node3.clone()];

        let strategy = ConsistentHashPlacement::new();
        let room_a = RoomId::from("room-stream-alpha");
        let room_b = RoomId::from("room-stream-beta");

        // Placing room_a multiple times must deterministically yield the exact same node
        let pick1 = strategy.select_node(&room_a, &candidates).unwrap();
        let pick2 = strategy.select_node(&room_a, &candidates).unwrap();
        assert_eq!(pick1.id, pick2.id);

        let pick_b1 = strategy.select_node(&room_b, &candidates).unwrap();
        let pick_b2 = strategy.select_node(&room_b, &candidates).unwrap();
        assert_eq!(pick_b1.id, pick_b2.id);

        // When the selected node drains, placement gracefully shifts to an available peer
        pick1.set_status(NodeStatus::Draining);
        let failover_pick = strategy.select_node(&room_a, &candidates).unwrap();
        assert_ne!(failover_pick.id, pick1.id);
        assert!(failover_pick.is_available());
    }

    #[test]
    fn test_least_loaded_placement() {
        let node1 = Arc::new(ClusterNode::new("node-1", "http://node1:7880", "node1:7882", "us-east", 100));
        let node2 = Arc::new(ClusterNode::new("node-2", "http://node2:7880", "node2:7882", "us-east", 100));
        node1.active_rooms.store(80, Ordering::Relaxed);
        node2.active_rooms.store(10, Ordering::Relaxed);

        let candidates = vec![node1.clone(), node2.clone()];
        let strategy = LeastLoadedPlacement::new();

        let room = RoomId::from("new-room");
        let selected = strategy.select_node(&room, &candidates).unwrap();
        assert_eq!(selected.id, "node-2", "Must pick the node with lowest load factor");
    }
}
