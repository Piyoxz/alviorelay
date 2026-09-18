use alvio_core::{PeerId, RoomId, WebhooksConfig};
use alvio_hooks::{WebhookDispatcher, WebhookEvent, WebhookSigner};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Router,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

struct ReceivedPayload {
    event_id: String,
    timestamp: u64,
    signature: String,
    body: Vec<u8>,
}

#[derive(Clone)]
struct MockReceiverState {
    received: Arc<Mutex<Vec<ReceivedPayload>>>,
    fail_first_n_times: Arc<AtomicUsize>,
    signer: WebhookSigner,
}

async fn mock_webhook_handler(
    State(state): State<MockReceiverState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let fail_count = state.fail_first_n_times.load(Ordering::Relaxed);
    if fail_count > 0 {
        state.fail_first_n_times.fetch_sub(1, Ordering::Relaxed);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Simulated transient 500");
    }

    let event_id = headers
        .get("alvio-event-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let timestamp: u64 = headers
        .get("alvio-timestamp")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or_default();

    let signature = headers
        .get("alvio-signature")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    // Verify HMAC-SHA256 signature
    let is_valid = state
        .signer
        .verify(timestamp, &body, &signature, Some(60))
        .unwrap_or(false);

    assert!(is_valid, "HMAC-SHA256 signature must be cryptographically valid");

    state.received.lock().await.push(ReceivedPayload {
        event_id,
        timestamp,
        signature,
        body: body.to_vec(),
    });

    (StatusCode::OK, "OK")
}

#[tokio::test]
async fn test_webhook_dispatch_and_hmac_verification() {
    let secret = "top_secret_hmac_key_987";
    let signer = WebhookSigner::new(secret).unwrap();
    let received_store = Arc::new(Mutex::new(Vec::new()));

    let mock_state = MockReceiverState {
        received: Arc::clone(&received_store),
        fail_first_n_times: Arc::new(AtomicUsize::new(0)),
        signer,
    };

    let app = Router::new()
        .route("/webhooks/alvio", post(mock_webhook_handler))
        .with_state(mock_state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let endpoint_url = format!("http://{addr}/webhooks/alvio");

    let config = WebhooksConfig {
        enabled: true,
        endpoint_url: Some(endpoint_url),
        secret: Some(secret.to_string()),
    };

    let dispatcher = WebhookDispatcher::new(&config);
    assert!(dispatcher.is_enabled());

    let room_id = RoomId::from("auditorium-1");
    let peer_id = PeerId::from("speaker-bob");

    // Dispatch 2 events
    dispatcher
        .dispatch(WebhookEvent::room_created(&room_id))
        .unwrap();

    dispatcher
        .dispatch(WebhookEvent::peer_joined(&room_id, &peer_id, "Bob", Some("presenter")))
        .unwrap();

    // Allow background worker to deliver
    tokio::time::sleep(Duration::from_millis(200)).await;

    let items = received_store.lock().await;
    assert_eq!(items.len(), 2);

    // Verify first event payload content
    let first_json: serde_json::Value = serde_json::from_slice(&items[0].body).unwrap();
    assert_eq!(first_json["event"], "room_created");
    assert_eq!(first_json["payload"]["room_id"], "auditorium-1");

    // Verify second event payload content
    let second_json: serde_json::Value = serde_json::from_slice(&items[1].body).unwrap();
    assert_eq!(second_json["event"], "peer_joined");
    assert_eq!(second_json["payload"]["peer_id"], "speaker-bob");

    assert_eq!(dispatcher.metrics().events_delivered.load(Ordering::Relaxed), 2);
    assert!(items[0].event_id.starts_with("evt_"));
    assert!(items[0].timestamp > 0);
    assert!(items[0].signature.starts_with("sha256="));
}

#[tokio::test]
async fn test_webhook_retry_on_transient_failure() {
    let secret = "retry_test_secret_key";
    let signer = WebhookSigner::new(secret).unwrap();
    let received_store = Arc::new(Mutex::new(Vec::new()));

    // Simulate 1 initial 500 error before succeeding
    let mock_state = MockReceiverState {
        received: Arc::clone(&received_store),
        fail_first_n_times: Arc::new(AtomicUsize::new(1)),
        signer,
    };

    let app = Router::new()
        .route("/webhooks/retry", post(mock_webhook_handler))
        .with_state(mock_state);

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let endpoint_url = format!("http://{addr}/webhooks/retry");

    let config = WebhooksConfig {
        enabled: true,
        endpoint_url: Some(endpoint_url),
        secret: Some(secret.to_string()),
    };

    let dispatcher = WebhookDispatcher::new(&config);

    let room_id = RoomId::from("retry-room-1");
    dispatcher
        .dispatch(WebhookEvent::room_created(&room_id))
        .unwrap();

    // Allow time for 1 retry backoff (50ms)
    tokio::time::sleep(Duration::from_millis(300)).await;

    let items = received_store.lock().await;
    assert_eq!(items.len(), 1, "Should have succeeded after retry");
    assert_eq!(dispatcher.metrics().events_delivered.load(Ordering::Relaxed), 1);
}
