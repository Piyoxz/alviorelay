use thiserror::Error;

pub type ClusterResult<T> = Result<T, ClusterError>;

#[derive(Debug, Error)]
pub enum ClusterError {
    #[error("Node '{0}' not found in cluster registry")]
    NodeNotFound(String),

    #[error("Node '{0}' is currently in drain mode and refusing new allocations")]
    NodeDraining(String),

    #[error("No healthy active nodes available in the cluster")]
    NoHealthyNodes,

    #[error("Failed to place room onto cluster: {0}")]
    PlacementFailed(String),

    #[error("Node '{0}' is already registered in the cluster")]
    AlreadyRegistered(String),

    #[error("Cluster drain timeout exceeded: {0}s")]
    DrainTimeout(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_error_display() {
        let err = ClusterError::NodeNotFound("node-us-east-1".into());
        assert!(err.to_string().contains("node-us-east-1"));

        let no_nodes = ClusterError::NoHealthyNodes;
        assert_eq!(
            no_nodes.to_string(),
            "No healthy active nodes available in the cluster"
        );
    }
}
