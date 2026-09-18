use crate::error::{WebhookError, WebhookResult};
use crate::event::current_timestamp;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Cryptographic signer and validator for webhook HTTP request payloads using HMAC-SHA256.
#[derive(Clone)]
pub struct WebhookSigner {
    secret: String,
}

impl WebhookSigner {
    pub const DEFAULT_TOLERANCE_SECS: u64 = 300; // 5 minutes

    pub fn new(secret: impl Into<String>) -> WebhookResult<Self> {
        let secret = secret.into();
        if secret.trim().is_empty() {
            return Err(WebhookError::InvalidSecret);
        }
        Ok(Self { secret })
    }

    /// Computes the HMAC-SHA256 signature string for `{timestamp}.{payload}`.
    ///
    /// Returns the formatted header value: `sha256={hex_digest}`.
    pub fn sign(&self, timestamp: u64, payload: &[u8]) -> WebhookResult<String> {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| WebhookError::SigningError(e.to_string()))?;

        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);

        let result = mac.finalize();
        let hex_signature = hex::encode(result.into_bytes());
        Ok(format!("sha256={hex_signature}"))
    }

    /// Validates an incoming webhook signature and ensures the timestamp is within tolerance.
    pub fn verify(
        &self,
        timestamp: u64,
        payload: &[u8],
        signature_header: &str,
        tolerance_secs: Option<u64>,
    ) -> WebhookResult<bool> {
        if let Some(tolerance) = tolerance_secs {
            let now = current_timestamp();
            let diff = now.abs_diff(timestamp);
            if diff > tolerance {
                return Err(WebhookError::TimestampExpired(diff));
            }
        }

        let sig_hex = signature_header
            .strip_prefix("sha256=")
            .unwrap_or(signature_header);

        let sig_bytes = hex::decode(sig_hex).map_err(|_| WebhookError::InvalidSignature)?;

        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .map_err(|e| WebhookError::SigningError(e.to_string()))?;

        mac.update(timestamp.to_string().as_bytes());
        mac.update(b".");
        mac.update(payload);

        mac.verify_slice(&sig_bytes)
            .map_err(|_| WebhookError::InvalidSignature)?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_signature_generation_and_verification() {
        let signer = WebhookSigner::new("my_super_secret_key_123").unwrap();
        let timestamp = current_timestamp();
        let payload = br#"{"event":"room.created","room_id":"livestream-1"}"#;

        let signature = signer.sign(timestamp, payload).unwrap();
        assert!(signature.starts_with("sha256="));

        // Valid signature passes verification
        let is_valid = signer
            .verify(timestamp, payload, &signature, Some(300))
            .unwrap();
        assert!(is_valid);

        // Tampered payload fails verification
        let tampered = br#"{"event":"room.created","room_id":"tampered-room"}"#;
        let verify_res = signer.verify(timestamp, tampered, &signature, Some(300));
        assert!(matches!(verify_res, Err(WebhookError::InvalidSignature)));

        // Tampered timestamp fails verification
        let wrong_time = timestamp + 1;
        let verify_res2 = signer.verify(wrong_time, payload, &signature, Some(300));
        assert!(matches!(verify_res2, Err(WebhookError::InvalidSignature)));

        // Expired timestamp fails replay protection
        let expired_time = timestamp - 1000;
        let expired_sig = signer.sign(expired_time, payload).unwrap();
        let replay_res = signer.verify(expired_time, payload, &expired_sig, Some(60));
        assert!(matches!(replay_res, Err(WebhookError::TimestampExpired(_))));
    }
}
