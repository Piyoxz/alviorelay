use thiserror::Error;

pub type WebhookResult<T> = Result<T, WebhookError>;

#[derive(Debug, Error)]
pub enum WebhookError {
    #[error("Invalid or empty webhook secret")]
    InvalidSecret,

    #[error("Cryptographic signing error: {0}")]
    SigningError(String),

    #[error("Event payload serialization error: {0}")]
    SerializationError(String),

    #[error("Webhook delivery failed after max retries: {0}")]
    DeliveryFailed(String),

    #[error("Webhook event queue is full, dropping event to prevent SFU pipeline stalling")]
    QueueFull,

    #[error("Invalid webhook HMAC-SHA256 signature")]
    InvalidSignature,

    #[error("Webhook timestamp expired or clock skew exceeded tolerance: difference={0}s")]
    TimestampExpired(u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_error_formatting() {
        let err = WebhookError::DeliveryFailed("connection refused".into());
        assert!(err.to_string().contains("connection refused"));

        let queue_err = WebhookError::QueueFull;
        assert!(queue_err.to_string().contains("queue is full"));
    }
}
