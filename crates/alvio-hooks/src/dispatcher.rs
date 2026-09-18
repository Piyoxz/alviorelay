use crate::error::{WebhookError, WebhookResult};
use crate::event::WebhookEvent;
use crate::signer::WebhookSigner;
use alvio_core::WebhooksConfig;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

/// Non-blocking, fault-tolerant Webhook Dispatcher.
///
/// Buffers events via a bounded queue and dispatches HTTP POST notifications with
/// HMAC-SHA256 signatures, replay-safe timestamps, and exponential backoff retry.
#[derive(Clone)]
pub struct WebhookDispatcher {
    tx: Option<mpsc::Sender<WebhookEvent>>,
    enabled: bool,
    metrics: Arc<DispatcherMetrics>,
}

#[derive(Default)]
pub struct DispatcherMetrics {
    pub events_dispatched: AtomicU64,
    pub events_delivered: AtomicU64,
    pub events_failed: AtomicU64,
    pub events_dropped: AtomicU64,
}

impl WebhookDispatcher {
    pub const DEFAULT_QUEUE_CAPACITY: usize = 4096;
    pub const MAX_RETRIES: u32 = 3;

    pub fn new(config: &WebhooksConfig) -> Self {
        let metrics = Arc::new(DispatcherMetrics::default());

        if !config.enabled || config.endpoint_url.is_none() {
            debug!("Webhooks are disabled or no endpoint URL configured");
            return Self {
                tx: None,
                enabled: false,
                metrics,
            };
        }

        let endpoint_url = config.endpoint_url.clone().unwrap();
        let signer = config
            .secret
            .as_ref()
            .and_then(|sec| WebhookSigner::new(sec).ok());

        let (tx, mut rx) = mpsc::channel::<WebhookEvent>(Self::DEFAULT_QUEUE_CAPACITY);
        let worker_metrics = Arc::clone(&metrics);

        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new());

            while let Some(event) = rx.recv().await {
                worker_metrics.events_dispatched.fetch_add(1, Ordering::Relaxed);
                let payload_bytes = match serde_json::to_vec(&event) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        error!(error = %e, "Failed to serialize webhook event payload");
                        worker_metrics.events_failed.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                };

                let signature = signer.as_ref().and_then(|s| s.sign(event.timestamp, &payload_bytes).ok());

                let mut delivered = false;
                for attempt in 0..Self::MAX_RETRIES {
                    let mut req = client
                        .post(&endpoint_url)
                        .header(reqwest::header::CONTENT_TYPE, "application/json")
                        .header(reqwest::header::USER_AGENT, "AlvioRelay-Webhooks/0.1.0")
                        .header("Alvio-Event-Id", &event.id)
                        .header("Alvio-Timestamp", event.timestamp.to_string())
                        .body(payload_bytes.clone());

                    if let Some(ref sig) = signature {
                        req = req.header("Alvio-Signature", sig);
                    }

                    match req.send().await {
                        Ok(resp) => {
                            if resp.status().is_success() {
                                debug!(
                                    event_id = %event.id,
                                    event = ?event.event,
                                    attempt,
                                    "Webhook delivered successfully"
                                );
                                worker_metrics.events_delivered.fetch_add(1, Ordering::Relaxed);
                                delivered = true;
                                break;
                            } else if resp.status().is_client_error() {
                                warn!(
                                    event_id = %event.id,
                                    status = %resp.status(),
                                    "Webhook rejected by client endpoint (4xx), not retrying"
                                );
                                break;
                            } else {
                                warn!(
                                    event_id = %event.id,
                                    status = %resp.status(),
                                    attempt,
                                    "Webhook endpoint returned 5xx, scheduling retry"
                                );
                            }
                        }
                        Err(e) => {
                            warn!(
                                event_id = %event.id,
                                error = %e,
                                attempt,
                                "Webhook delivery network error, scheduling retry"
                            );
                        }
                    }

                    // Exponential backoff: 50ms * 2^attempt
                    if attempt + 1 < Self::MAX_RETRIES {
                        let delay = Duration::from_millis(50 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                    }
                }

                if !delivered {
                    worker_metrics.events_failed.fetch_add(1, Ordering::Relaxed);
                    error!(
                        event_id = %event.id,
                        event = ?event.event,
                        "Webhook delivery permanently failed after max retries"
                    );
                }
            }
        });

        Self {
            tx: Some(tx),
            enabled: true,
            metrics,
        }
    }

    /// Dispatches an event into the asynchronous delivery queue without blocking.
    pub fn dispatch(&self, event: WebhookEvent) -> WebhookResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let Some(ref tx) = self.tx else {
            return Ok(());
        };

        match tx.try_send(event) {
            Ok(()) => Ok(()),
            Err(mpsc::error::TrySendError::Full(_)) => {
                self.metrics.events_dropped.fetch_add(1, Ordering::Relaxed);
                warn!("Webhook delivery queue is full! Dropping event to preserve media SFU throughput");
                Err(WebhookError::QueueFull)
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                self.metrics.events_failed.fetch_add(1, Ordering::Relaxed);
                Err(WebhookError::DeliveryFailed("Worker channel closed".to_string()))
            }
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn metrics(&self) -> &Arc<DispatcherMetrics> {
        &self.metrics
    }
}
