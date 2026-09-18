use crate::error::{ClusterError, ClusterResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

/// Controller managing node Graceful Drain Mode for zero-downtime rolling maintenance.
#[derive(Clone)]
pub struct DrainController {
    is_draining: Arc<AtomicBool>,
}

impl Default for DrainController {
    fn default() -> Self {
        Self::new()
    }
}

impl DrainController {
    pub fn new() -> Self {
        Self {
            is_draining: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Triggers the node to enter Drain Mode.
    ///
    /// Once draining:
    /// - Kubernetes readiness probes (`/ready`) will fail with HTTP 503.
    /// - New room creations will be rejected or diverted to other cluster nodes.
    /// - Existing active rooms will run without interruption until participants leave.
    pub fn start_drain(&self) -> bool {
        let already_draining = self.is_draining.swap(true, Ordering::SeqCst);
        if !already_draining {
            warn!("Node has entered Graceful Drain Mode: rejecting new allocations, awaiting active rooms to finish");
        }
        !already_draining
    }

    pub fn is_draining(&self) -> bool {
        self.is_draining.load(Ordering::Relaxed)
    }

    pub fn can_accept_new_room(&self) -> bool {
        !self.is_draining()
    }

    /// Asynchronously polls until the active room count reaches zero or a timeout occurs.
    pub async fn wait_until_drained<F>(
        &self,
        active_room_count_fn: F,
        poll_interval: Duration,
        timeout: Option<Duration>,
    ) -> ClusterResult<()>
    where
        F: Fn() -> usize,
    {
        let start = tokio::time::Instant::now();

        loop {
            let active = active_room_count_fn();
            if active == 0 {
                info!("Graceful drain completed successfully: 0 active rooms remaining");
                return Ok(());
            }

            if let Some(t) = timeout {
                if start.elapsed() >= t {
                    warn!(active_rooms = active, "Drain timeout exceeded before room count reached zero");
                    return Err(ClusterError::DrainTimeout(t.as_secs()));
                }
            }

            info!(active_rooms = active, "Drain in progress: waiting for active rooms to conclude...");
            tokio::time::sleep(poll_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[tokio::test]
    async fn test_drain_controller_lifecycle() {
        let controller = DrainController::new();
        assert!(!controller.is_draining());
        assert!(controller.can_accept_new_room());

        // Start drain
        assert!(controller.start_drain());
        assert!(controller.is_draining());
        assert!(!controller.can_accept_new_room());

        // Starting drain second time returns false (already draining)
        assert!(!controller.start_drain());

        // Simulate active room drain
        let rooms = Arc::new(AtomicUsize::new(2));
        let rooms_clone = Arc::clone(&rooms);

        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            rooms_clone.store(1, Ordering::Relaxed);
            tokio::time::sleep(Duration::from_millis(50)).await;
            rooms_clone.store(0, Ordering::Relaxed);
        });

        let rooms_counter = Arc::clone(&rooms);
        let result = controller
            .wait_until_drained(
                move || rooms_counter.load(Ordering::Relaxed),
                Duration::from_millis(20),
                Some(Duration::from_secs(1)),
            )
            .await;

        assert!(result.is_ok());
    }
}
