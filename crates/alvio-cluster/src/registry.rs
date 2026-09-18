use crate::error::{ClusterError, ClusterResult};
use crate::node::ClusterNode;
use crate::placement::{ConsistentHashPlacement, PlacementStrategy};
use alvio_core::RoomId;
use dashmap::DashMap;
use std::sync::Arc;

/// Global thread-safe registry of cluster nodes and authoritative room routing.
pub struct ClusterRegistry {
    nodes: DashMap<String, Arc<ClusterNode>>,
    placement: Arc<dyn PlacementStrategy>,
}

impl Default for ClusterRegistry {
    fn default() -> Self {
        Self::new(Arc::new(ConsistentHashPlacement::new()))
    }
}

impl ClusterRegistry {
    pub fn new(placement: Arc<dyn PlacementStrategy>) -> Self {
        Self {
            nodes: DashMap::new(),
            placement,
        }
    }

    /// Adds a node into the cluster topology.
    pub fn register(&self, node: ClusterNode) -> ClusterResult<Arc<ClusterNode>> {
        if self.nodes.contains_key(&node.id) {
            return Err(ClusterError::AlreadyRegistered(node.id));
        }

        let node_id = node.id.clone();
        let arc_node = Arc::new(node);
        self.nodes.insert(node_id, Arc::clone(&arc_node));
        Ok(arc_node)
    }

    /// Removes a node from the cluster.
    pub fn unregister(&self, node_id: &str) -> Option<Arc<ClusterNode>> {
        self.nodes.remove(node_id).map(|(_, n)| n)
    }

    /// Looks up a specific node by its identifier.
    pub fn get(&self, node_id: &str) -> Option<Arc<ClusterNode>> {
        self.nodes.get(node_id).map(|entry| Arc::clone(entry.value()))
    }

    /// Deterministically locates the authoritative node responsible for the given `RoomId`.
    pub fn locate_authoritative_node(&self, room_id: &RoomId) -> ClusterResult<Arc<ClusterNode>> {
        let candidates: Vec<Arc<ClusterNode>> = self
            .nodes
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        if candidates.is_empty() {
            return Err(ClusterError::NoHealthyNodes);
        }

        let selected = self.placement.select_node(room_id, &candidates)?;
        Ok(Arc::clone(selected))
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_registry_registration_and_lookup() {
        let registry = ClusterRegistry::default();
        let node1 = ClusterNode::new("node-alpha", "http://alpha:7880", "alpha:7882", "us-west", 100);
        let node2 = ClusterNode::new("node-beta", "http://beta:7880", "beta:7882", "eu-central", 100);

        registry.register(node1).unwrap();
        registry.register(node2).unwrap();
        assert_eq!(registry.node_count(), 2);

        let room = RoomId::from("global-townhall");
        let authoritative = registry.locate_authoritative_node(&room).unwrap();
        assert!(authoritative.id == "node-alpha" || authoritative.id == "node-beta");

        // Unregister node
        let removed = registry.unregister("node-alpha").unwrap();
        assert_eq!(removed.id, "node-alpha");
        assert_eq!(registry.node_count(), 1);
    }
}
