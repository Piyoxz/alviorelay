use alvio_core::{PeerId, RoomId, StreamKind, StreamLayer, TrackId};
use alvio_protocol::{PeerInfo, SignalEnvelope, SignalMessage, TrackInfo};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::error::ClientError;
use crate::event::ClientEvent;
use crate::signaling::SignalingTransport;
use crate::state::ConnectionState;

/// High-level client room instance managing connection, signaling, and room state.
#[derive(Clone)]
pub struct ClientRoom {
    client_version: String,
    state: Arc<RwLock<ConnectionState>>,
    self_peer_id: Arc<RwLock<Option<PeerId>>>,
    room_id: Arc<RwLock<Option<RoomId>>>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    tracks: Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
    event_tx: broadcast::Sender<ClientEvent>,
    outbound_tx: Arc<RwLock<Option<mpsc::Sender<SignalEnvelope>>>>,
    reader_task: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl ClientRoom {
    /// Create a new ClientRoom with the specified client version string.
    pub fn new(client_version: impl Into<String>) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            client_version: client_version.into(),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            self_peer_id: Arc::new(RwLock::new(None)),
            room_id: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            tracks: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            outbound_tx: Arc::new(RwLock::new(None)),
            reader_task: Arc::new(RwLock::new(None)),
        }
    }

    /// Current connection state.
    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    /// Assigned peer ID (if connected and acknowledged by server).
    pub fn self_peer_id(&self) -> Option<PeerId> {
        self.self_peer_id.read().clone()
    }

    /// Active room ID (if currently joined).
    pub fn room_id(&self) -> Option<RoomId> {
        self.room_id.read().clone()
    }

    /// List of remote peers currently in the room.
    pub fn peers(&self) -> Vec<PeerInfo> {
        self.peers.read().values().cloned().collect()
    }

    /// List of active tracks in the room.
    pub fn tracks(&self) -> Vec<TrackInfo> {
        self.tracks.read().values().cloned().collect()
    }

    /// Subscribe to the broadcast stream of client events.
    pub fn subscribe_events(&self) -> broadcast::Receiver<ClientEvent> {
        self.event_tx.subscribe()
    }

    /// Connect to the signaling gateway using the provided transport.
    pub async fn connect<T: SignalingTransport + 'static>(
        &self,
        mut transport: T,
        token: Option<String>,
    ) -> Result<(), ClientError> {
        if self.state() != ConnectionState::Disconnected {
            return Err(ClientError::AlreadyConnected);
        }

        self.set_state(ConnectionState::Connecting);

        let (outbound_tx, mut outbound_rx) = mpsc::channel::<SignalEnvelope>(128);
        *self.outbound_tx.write() = Some(outbound_tx.clone());

        let event_tx = self.event_tx.clone();
        let state_lock = self.state.clone();
        let self_peer_id_lock = self.self_peer_id.clone();
        let room_id_lock = self.room_id.clone();
        let peers_lock = self.peers.clone();
        let tracks_lock = self.tracks.clone();

        let reader = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(envelope) = outbound_rx.recv() => {
                        if let Err(e) = transport.send(envelope).await {
                            error!("Failed to send outbound signaling envelope: {}", e);
                            break;
                        }
                    }
                    incoming = transport.recv() => {
                        match incoming {
                            Ok(Some(envelope)) => {
                                Self::process_incoming_message(
                                    envelope.body,
                                    &event_tx,
                                    &state_lock,
                                    &self_peer_id_lock,
                                    &room_id_lock,
                                    &peers_lock,
                                    &tracks_lock,
                                );
                            }
                            Ok(None) => {
                                debug!("Signaling transport EOF, closing connection");
                                break;
                            }
                            Err(e) => {
                                error!("Signaling transport read error: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            *state_lock.write() = ConnectionState::Disconnected;
            let _ = event_tx.send(ClientEvent::StateChanged(ConnectionState::Disconnected));
            let _ = event_tx.send(ClientEvent::Disconnected);
            let _ = transport.close().await;
        });

        *self.reader_task.write() = Some(reader);

        let handshake = SignalEnvelope::new(SignalMessage::Connect {
            token,
            client_version: self.client_version.clone(),
        });
        outbound_tx
            .send(handshake)
            .await
            .map_err(|_| ClientError::ChannelClosed)?;

        Ok(())
    }

    /// Request to join a room.
    pub async fn join(
        &self,
        room_id: RoomId,
        peer_name: impl Into<String>,
        metadata: Option<String>,
    ) -> Result<(), ClientError> {
        let msg = SignalMessage::Join {
            room_id,
            peer_name: peer_name.into(),
            metadata,
        };
        self.send_message(msg).await
    }

    /// Publish a media track to the room.
    pub async fn publish_track(
        &self,
        kind: StreamKind,
        source: impl Into<String>,
        layers: Vec<StreamLayer>,
    ) -> Result<(), ClientError> {
        let msg = SignalMessage::PublishTrack {
            kind,
            source: source.into(),
            layers,
        };
        self.send_message(msg).await
    }

    /// Unpublish a media track.
    pub async fn unpublish_track(&self, track_id: TrackId) -> Result<(), ClientError> {
        let msg = SignalMessage::UnpublishTrack { track_id };
        self.send_message(msg).await
    }

    /// Subscribe to a remote track with optional initial preferred layer.
    pub async fn subscribe(
        &self,
        track_id: TrackId,
        preferred_layer: Option<StreamLayer>,
    ) -> Result<(), ClientError> {
        let msg = SignalMessage::Subscribe {
            track_id,
            preferred_layer,
        };
        self.send_message(msg).await
    }

    /// Change preferred layer for an active simulcast subscription.
    pub async fn select_layer(
        &self,
        track_id: TrackId,
        layer: StreamLayer,
    ) -> Result<(), ClientError> {
        let msg = SignalMessage::LayerSelect { track_id, layer };
        self.send_message(msg).await
    }

    /// Send an application data message to target peers or broadcast (empty list = room broadcast).
    pub async fn send_data_message(
        &self,
        destination_peer_ids: Vec<PeerId>,
        payload: impl Into<String>,
        reliable: bool,
    ) -> Result<(), ClientError> {
        let msg = SignalMessage::DataMessage {
            destination_peer_ids,
            payload: payload.into(),
            reliable,
        };
        self.send_message(msg).await
    }

    /// Send WebRTC SDP offer.
    pub async fn send_sdp_offer(&self, sdp: impl Into<String>) -> Result<(), ClientError> {
        let msg = SignalMessage::Offer { sdp: sdp.into() };
        self.send_message(msg).await
    }

    /// Send WebRTC SDP answer.
    pub async fn send_sdp_answer(&self, sdp: impl Into<String>) -> Result<(), ClientError> {
        let msg = SignalMessage::Answer { sdp: sdp.into() };
        self.send_message(msg).await
    }

    /// Send WebRTC trickle ICE candidate.
    pub async fn send_candidate(
        &self,
        candidate: impl Into<String>,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u32>,
    ) -> Result<(), ClientError> {
        let msg = SignalMessage::Candidate {
            candidate: candidate.into(),
            sdp_mid,
            sdp_mline_index,
        };
        self.send_message(msg).await
    }

    /// Voluntarily leave the room and reset active state.
    pub async fn leave(&self) -> Result<(), ClientError> {
        let msg = SignalMessage::Leave;
        let _ = self.send_message(msg).await;

        *self.room_id.write() = None;
        self.peers.write().clear();
        self.tracks.write().clear();
        self.set_state(ConnectionState::Connected);

        Ok(())
    }

    /// Disconnect from the server and terminate background pump.
    pub async fn disconnect(&self) {
        if let Some(handle) = self.reader_task.write().take() {
            handle.abort();
        }
        *self.outbound_tx.write() = None;
        *self.self_peer_id.write() = None;
        *self.room_id.write() = None;
        self.peers.write().clear();
        self.tracks.write().clear();
        self.set_state(ConnectionState::Disconnected);
        let _ = self.event_tx.send(ClientEvent::Disconnected);
    }

    async fn send_message(&self, message: SignalMessage) -> Result<(), ClientError> {
        let tx = {
            let guard = self.outbound_tx.read();
            guard.clone().ok_or(ClientError::NotConnected)?
        };

        let envelope = SignalEnvelope::new(message);
        tx.send(envelope)
            .await
            .map_err(|_| ClientError::ChannelClosed)?;

        Ok(())
    }

    fn set_state(&self, state: ConnectionState) {
        *self.state.write() = state;
        let _ = self.event_tx.send(ClientEvent::StateChanged(state));
    }

    fn process_incoming_message(
        msg: SignalMessage,
        event_tx: &broadcast::Sender<ClientEvent>,
        state_lock: &Arc<RwLock<ConnectionState>>,
        self_peer_id_lock: &Arc<RwLock<Option<PeerId>>>,
        room_id_lock: &Arc<RwLock<Option<RoomId>>>,
        peers_lock: &Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        tracks_lock: &Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
    ) {
        match msg {
            SignalMessage::Ack { peer_id, node_id } => {
                info!(
                    "Connected and acknowledged by node {} as {}",
                    node_id, peer_id
                );
                *self_peer_id_lock.write() = Some(peer_id.clone());
                *state_lock.write() = ConnectionState::Connected;
                let _ = event_tx.send(ClientEvent::StateChanged(ConnectionState::Connected));
                let _ = event_tx.send(ClientEvent::Connected { peer_id, node_id });
            }
            SignalMessage::RoomJoined {
                room_id,
                self_peer_id,
                peers,
                active_tracks,
            } => {
                info!("Joined room {} as peer {}", room_id, self_peer_id);
                *room_id_lock.write() = Some(room_id.clone());
                *self_peer_id_lock.write() = Some(self_peer_id.clone());

                let mut p_map = peers_lock.write();
                p_map.clear();
                for p in &peers {
                    p_map.insert(p.id.clone(), p.clone());
                }

                let mut t_map = tracks_lock.write();
                t_map.clear();
                for t in &active_tracks {
                    t_map.insert(t.id.clone(), t.clone());
                }

                *state_lock.write() = ConnectionState::InRoom;
                let _ = event_tx.send(ClientEvent::StateChanged(ConnectionState::InRoom));
                let _ = event_tx.send(ClientEvent::RoomJoined {
                    room_id,
                    self_peer_id,
                    peers,
                    active_tracks,
                });
            }
            SignalMessage::PeerJoined { peer } => {
                peers_lock.write().insert(peer.id.clone(), peer.clone());
                let _ = event_tx.send(ClientEvent::PeerJoined(peer));
            }
            SignalMessage::PeerLeft { peer_id, reason } => {
                peers_lock.write().remove(&peer_id);
                let _ = event_tx.send(ClientEvent::PeerLeft { peer_id, reason });
            }
            SignalMessage::TrackPublished { track } => {
                tracks_lock.write().insert(track.id.clone(), track.clone());
                let _ = event_tx.send(ClientEvent::TrackPublished(track));
            }
            SignalMessage::TrackUnpublished { track_id } => {
                tracks_lock.write().remove(&track_id);
                let _ = event_tx.send(ClientEvent::TrackUnpublished(track_id));
            }
            SignalMessage::DataReceived {
                source_peer_id,
                payload,
            } => {
                let _ = event_tx.send(ClientEvent::DataReceived {
                    source_peer_id,
                    payload,
                });
            }
            SignalMessage::RemoteOffer { sdp } => {
                let _ = event_tx.send(ClientEvent::RemoteOffer { sdp });
            }
            SignalMessage::RemoteAnswer { sdp } => {
                let _ = event_tx.send(ClientEvent::RemoteAnswer { sdp });
            }
            SignalMessage::RemoteCandidate {
                candidate,
                sdp_mid,
                sdp_mline_index,
            } => {
                let _ = event_tx.send(ClientEvent::RemoteCandidate {
                    candidate,
                    sdp_mid,
                    sdp_mline_index,
                });
            }
            SignalMessage::Error { code, message } => {
                warn!("Received error from server: [{}] {}", code, message);
                let _ = event_tx.send(ClientEvent::Error { code, message });
            }
            SignalMessage::Pong => {
                debug!("Received server pong heartbeat");
            }
            _ => {
                debug!("Ignoring unhandled client incoming message variant");
            }
        }
    }
}
