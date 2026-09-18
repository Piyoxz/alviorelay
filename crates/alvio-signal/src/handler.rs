use crate::registry::{PeerSession, RoomRegistry, RoomSession};
use alvio_core::{PeerId, TrackId};
use alvio_protocol::{SignalEnvelope, SignalMessage, TrackInfo};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct SignalState {
    pub registry: RoomRegistry,
    pub node_id: String,
}

/// Upgrades HTTP request to WebSocket signaling connection.
pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    State(state): State<SignalState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SignalState) {
    let (mut ws_sender, mut ws_receiver) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<SignalEnvelope>();

    let peer_id = PeerId::new(format!("peer_{}", Uuid::new_v4().simple()));
    info!(peer = %peer_id, "New signaling WebSocket connection established");

    let ack_env = SignalEnvelope::new(SignalMessage::Ack {
        peer_id: peer_id.clone(),
        node_id: state.node_id.clone(),
    });
    if let Ok(json) = ack_env.to_json() {
        let _ = ws_sender.send(Message::Text(json.into())).await;
    }

    let outbound_peer_id = peer_id.clone();
    let outbound_task = tokio::spawn(async move {
        while let Some(envelope) = rx.recv().await {
            match envelope.to_json() {
                Ok(json) => {
                    if ws_sender.send(Message::Text(json.into())).await.is_err() {
                        debug!(peer = %outbound_peer_id, "Failed to send WS message, client disconnected");
                        break;
                    }
                }
                Err(e) => {
                    error!(peer = %outbound_peer_id, error = %e, "Failed to serialize signaling envelope");
                }
            }
        }
    });

    let mut current_room: Option<Arc<RoomSession>> = None;
    let mut peer_name = "Anonymous".to_string();

    while let Some(msg_result) = ws_receiver.next().await {
        match msg_result {
            Ok(Message::Text(text)) => match SignalEnvelope::from_json(&text) {
                Ok(envelope) => {
                    handle_client_message(
                        &peer_id,
                        &mut peer_name,
                        &envelope.body,
                        &tx,
                        &state,
                        &mut current_room,
                    )
                    .await;
                }
                Err(e) => {
                    warn!(peer = %peer_id, error = %e, "Invalid JSON received from client");
                    let _ = tx.send(SignalEnvelope::new(SignalMessage::Error {
                        code: "INVALID_JSON".to_string(),
                        message: format!("Malformed signaling payload: {e}"),
                    }));
                }
            },
            Ok(Message::Ping(payload)) => {
                let _ = tx.send(SignalEnvelope::new(SignalMessage::Pong));
                debug!(peer = %peer_id, len = payload.len(), "Received WS Ping");
            }
            Ok(Message::Close(_)) => {
                info!(peer = %peer_id, "Client sent Close frame");
                break;
            }
            Err(e) => {
                warn!(peer = %peer_id, error = %e, "WebSocket stream error");
                break;
            }
            _ => {}
        }
    }

    if let Some(room) = current_room.take() {
        room.leave_peer(&peer_id, "disconnected");
    }

    outbound_task.abort();
    info!(peer = %peer_id, "Signaling connection cleanly terminated");
}

async fn handle_client_message(
    peer_id: &PeerId,
    peer_name: &mut String,
    message: &SignalMessage,
    tx: &mpsc::UnboundedSender<SignalEnvelope>,
    state: &SignalState,
    current_room: &mut Option<Arc<RoomSession>>,
) {
    match message {
        SignalMessage::Connect { .. } => {
            debug!(peer = %peer_id, "Received Connect handshake message");
        }
        SignalMessage::Join {
            room_id,
            peer_name: name,
            metadata,
        } => {
            *peer_name = name.clone();
            let room = state.registry.get_or_create(room_id);

            let peer_session = Arc::new(PeerSession::new(
                peer_id.clone(),
                name.clone(),
                metadata.clone(),
                tx.clone(),
            ));

            match room.join_peer(peer_session) {
                Ok(()) => {
                    info!(peer = %peer_id, name = %name, room = %room_id, "Peer joined room successfully");
                    let (peers, active_tracks) = room.get_snapshot();

                    let _ = tx.send(SignalEnvelope::new(SignalMessage::RoomJoined {
                        room_id: room_id.clone(),
                        self_peer_id: peer_id.clone(),
                        peers,
                        active_tracks,
                    }));

                    *current_room = Some(room);
                }
                Err(e) => {
                    warn!(peer = %peer_id, room = %room_id, error = %e, "Failed to join room");
                    let _ = tx.send(SignalEnvelope::new(SignalMessage::Error {
                        code: e.code().to_string(),
                        message: e.to_string(),
                    }));
                }
            }
        }
        SignalMessage::PublishTrack {
            kind,
            source,
            layers,
        } => {
            if let Some(room) = current_room.as_ref() {
                let track_id = TrackId::new(format!("trk_{}", Uuid::new_v4().simple()));
                let track = TrackInfo {
                    id: track_id.clone(),
                    peer_id: peer_id.clone(),
                    kind: *kind,
                    source: source.clone(),
                    layers: layers.clone(),
                };

                room.publish_track(track.clone());

                let _ = tx.send(SignalEnvelope::new(SignalMessage::TrackPublished { track }));
                info!(peer = %peer_id, track = %track_id, kind = %kind, "Track published");
            } else {
                let _ = tx.send(SignalEnvelope::new(SignalMessage::Error {
                    code: "NOT_IN_ROOM".to_string(),
                    message: "Cannot publish track when not in a room".to_string(),
                }));
            }
        }
        SignalMessage::UnpublishTrack { track_id } => {
            if let Some(room) = current_room.as_ref() {
                room.unpublish_track(track_id);
            }
        }
        SignalMessage::Offer { sdp } => {
            if let Some(room) = current_room.as_ref() {
                room.broadcast(
                    Some(peer_id),
                    SignalMessage::RemoteOffer { sdp: sdp.clone() },
                );
            }
        }
        SignalMessage::Answer { sdp } => {
            if let Some(room) = current_room.as_ref() {
                room.broadcast(
                    Some(peer_id),
                    SignalMessage::RemoteAnswer { sdp: sdp.clone() },
                );
            }
        }
        SignalMessage::Candidate {
            candidate,
            sdp_mid,
            sdp_mline_index,
        } => {
            if let Some(room) = current_room.as_ref() {
                room.broadcast(
                    Some(peer_id),
                    SignalMessage::RemoteCandidate {
                        candidate: candidate.clone(),
                        sdp_mid: sdp_mid.clone(),
                        sdp_mline_index: *sdp_mline_index,
                    },
                );
            }
        }
        SignalMessage::DataMessage {
            destination_peer_ids,
            payload,
            ..
        } => {
            if let Some(room) = current_room.as_ref() {
                room.send_direct_data(peer_id, destination_peer_ids, payload);
            }
        }
        SignalMessage::Leave => {
            if let Some(room) = current_room.take() {
                room.leave_peer(peer_id, "voluntary_leave");
            }
        }
        SignalMessage::Ping => {
            let _ = tx.send(SignalEnvelope::new(SignalMessage::Pong));
        }
        _ => {}
    }
}
