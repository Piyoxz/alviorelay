use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Operational state of a node in the AlvioRelay cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    /// Actively accepting new room assignments and client connections.
    Active,
    /// Refusing new room allocations; existing rooms continue until empty.
    Draining,
    /// Node is unreachable or decommissioned.
    Offline,
}

/// Metadata and real-time load state of a cluster instance.
#[derive(Debug)]
pub struct ClusterNode {
    pub id: String,
    pub http_url: String,
    pub rtc_address: String,
    pub region: String,
    pub max_rooms: usize,
    pub active_rooms: AtomicUsize,
    pub active_peers: AtomicUsize,
    pub status: Arc<RwLock<NodeStatus>>,
    pub last_heartbeat: AtomicU64,
}

impl ClusterNode {
    pub fn new(
        id: impl Into<String>,
        http_url: impl Into<String>,
        rtc_address: impl Into<String>,
        region: impl Into<String>,
        max_rooms: usize,
    ) -> Self {
        Self {
            id: id.into(),
            http_url: http_url.into(),
            rtc_address: rtc_address.into(),
            region: region.into(),
            max_rooms,
            active_rooms: AtomicUsize::new(0),
            active_peers: AtomicUsize::new(0),
            status: Arc::new(RwLock::new(NodeStatus::Active)),
            last_heartbeat: AtomicU64::new(current_timestamp()),
        }
    }

    /// Determines if this node is eligible to host a newly created room.
    pub fn is_available(&self) -> bool {
        *self.status.read() == NodeStatus::Active
            && self.active_rooms.load(Ordering::Relaxed) < self.max_rooms
    }

    /// Returns the current load fraction (0.0 to 1.0).
    pub fn load_factor(&self) -> f64 {
        let active = self.active_rooms.load(Ordering::Relaxed);
        if self.max_rooms == 0 {
            1.0
        } else {
            active as f64 / self.max_rooms as f64
        }
    }

    pub fn set_status(&self, new_status: NodeStatus) {
        *self.status.write() = new_status;
    }

    pub fn status(&self) -> NodeStatus {
        *self.status.read()
    }

    pub fn heartbeat(&self) {
        self.last_heartbeat
            .store(current_timestamp(), Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_node_lifecycle_and_load() {
        let node = ClusterNode::new(
            "node-singapore-1",
            "http://sg1.alvio.io:7880",
            "sg1.alvio.io:7882",
            "ap-southeast-1",
            100,
        );

        assert!(node.is_available());
        assert_eq!(node.status(), NodeStatus::Active);
        assert_eq!(node.load_factor(), 0.0);

        // Simulate room additions
        node.active_rooms.store(50, Ordering::Relaxed);
        assert_eq!(node.load_factor(), 0.5);

        // Drain node
        node.set_status(NodeStatus::Draining);
        assert_eq!(node.status(), NodeStatus::Draining);
        assert!(
            !node.is_available(),
            "Draining node must not be available for new rooms"
        );
    }
}
