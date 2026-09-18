use async_trait::async_trait;
use alvio_protocol::SignalEnvelope;
use tokio::sync::mpsc;
use crate::error::ClientError;

/// Asynchronous transport abstraction for client signaling communication.
#[async_trait]
pub trait SignalingTransport: Send + Sync {
    /// Send an outgoing signaling envelope to the server.
    async fn send(&self, envelope: SignalEnvelope) -> Result<(), ClientError>;

    /// Receive the next incoming signaling envelope from the server.
    async fn recv(&mut self) -> Result<Option<SignalEnvelope>, ClientError>;

    /// Gracefully close the signaling connection.
    async fn close(&mut self) -> Result<(), ClientError>;
}

/// In-memory mock transport pair for unit testing, simulations, and headless harnesses.
pub struct MockSignalingTransport {
    tx: mpsc::Sender<SignalEnvelope>,
    rx: mpsc::Receiver<SignalEnvelope>,
}

impl MockSignalingTransport {
    /// Create a connected client-server pair over async bounded channels.
    pub fn create_pair(buffer_size: usize) -> (Self, MockPeerServer) {
        let (client_tx, server_rx) = mpsc::channel(buffer_size);
        let (server_tx, client_rx) = mpsc::channel(buffer_size);

        let client = Self {
            tx: client_tx,
            rx: client_rx,
        };
        let server = MockPeerServer {
            tx: server_tx,
            rx: server_rx,
        };

        (client, server)
    }
}

#[async_trait]
impl SignalingTransport for MockSignalingTransport {
    async fn send(&self, envelope: SignalEnvelope) -> Result<(), ClientError> {
        self.tx
            .send(envelope)
            .await
            .map_err(|_| ClientError::ChannelClosed)
    }

    async fn recv(&mut self) -> Result<Option<SignalEnvelope>, ClientError> {
        Ok(self.rx.recv().await)
    }

    async fn close(&mut self) -> Result<(), ClientError> {
        self.rx.close();
        Ok(())
    }
}

/// Companion server-side harness to receive client messages and inject server events.
pub struct MockPeerServer {
    tx: mpsc::Sender<SignalEnvelope>,
    rx: mpsc::Receiver<SignalEnvelope>,
}

impl MockPeerServer {
    pub async fn send_to_client(&self, envelope: SignalEnvelope) -> Result<(), ClientError> {
        self.tx
            .send(envelope)
            .await
            .map_err(|_| ClientError::ChannelClosed)
    }

    pub async fn recv_from_client(&mut self) -> Option<SignalEnvelope> {
        self.rx.recv().await
    }
}
