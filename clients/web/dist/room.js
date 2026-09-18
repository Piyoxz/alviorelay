"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.AlvioRoom = void 0;
const signaling_js_1 = require("./signaling.js");
class AlvioRoom {
    signaling;
    state = 'disconnected';
    selfPeerId = null;
    roomId = null;
    peers = new Map();
    tracks = new Map();
    localTracks = new Map();
    listeners = new Map();
    constructor() {
        this.signaling = new signaling_js_1.SignalingClient();
        this.setupSignalingListeners();
    }
    getState() {
        return this.state;
    }
    getSelfPeerId() {
        return this.selfPeerId;
    }
    getRoomId() {
        return this.roomId;
    }
    getPeers() {
        return Array.from(this.peers.values());
    }
    getTracks() {
        return Array.from(this.tracks.values());
    }
    on(event, handler) {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, new Set());
        }
        this.listeners.get(event).add(handler);
    }
    off(event, handler) {
        const set = this.listeners.get(event);
        if (set) {
            set.delete(handler);
        }
    }
    emit(event, data) {
        const handlers = this.listeners.get(event);
        if (handlers) {
            for (const h of handlers) {
                try {
                    h(data);
                }
                catch (err) {
                    console.error(`[AlvioRoom] Event error '${event}':`, err);
                }
            }
        }
    }
    setState(newState) {
        this.state = newState;
        this.emit('stateChanged', newState);
    }
    setupSignalingListeners() {
        this.signaling.on('ack', (payload) => {
            this.selfPeerId = payload.peer_id;
            this.setState('connected');
            this.emit('connected', { peerId: payload.peer_id, nodeId: payload.node_id });
        });
        this.signaling.on('room_joined', (payload) => {
            this.roomId = payload.room_id;
            this.selfPeerId = payload.self_peer_id;
            this.peers.clear();
            for (const p of payload.peers || []) {
                this.peers.set(p.id, p);
            }
            this.tracks.clear();
            for (const t of payload.active_tracks || []) {
                this.tracks.set(t.id, t);
            }
            this.setState('in_room');
            this.emit('roomJoined', {
                roomId: payload.room_id,
                peers: this.getPeers(),
                tracks: this.getTracks(),
            });
        });
        this.signaling.on('peer_joined', (payload) => {
            const peer = payload.peer;
            if (peer) {
                this.peers.set(peer.id, peer);
                this.emit('peerJoined', peer);
            }
        });
        this.signaling.on('peer_left', (payload) => {
            if (payload.peer_id) {
                this.peers.delete(payload.peer_id);
                this.emit('peerLeft', { peerId: payload.peer_id, reason: payload.reason });
            }
        });
        this.signaling.on('track_published', (payload) => {
            const track = payload.track;
            if (track) {
                this.tracks.set(track.id, track);
                this.emit('trackPublished', track);
            }
        });
        this.signaling.on('track_unpublished', (payload) => {
            if (payload.track_id) {
                this.tracks.delete(payload.track_id);
                this.emit('trackUnpublished', payload.track_id);
            }
        });
        this.signaling.on('data_received', (payload) => {
            this.emit('dataReceived', {
                sourcePeerId: payload.source_peer_id,
                payload: payload.payload,
            });
        });
        this.signaling.on('error', (payload) => {
            this.emit('error', payload);
        });
        this.signaling.on('close', () => {
            this.setState('disconnected');
            this.emit('disconnected');
        });
        this.signaling.on('reconnecting', (data) => {
            this.setState('reconnecting');
            this.emit('reconnecting', data);
        });
    }
    async connect(url, token) {
        this.setState('connecting');
        await this.signaling.connect(url, token);
    }
    async join(roomId, peerName, metadata) {
        this.signaling.send('join', {
            room_id: roomId,
            peer_name: peerName,
            metadata,
        });
    }
    async publishTrack(track, options) {
        const kind = track.kind === 'video' ? 'video' : 'audio';
        const source = options?.source || (kind === 'video' ? 'camera' : 'microphone');
        const layers = options?.layers || (kind === 'video' && options?.simulcast
            ? ['low', 'medium', 'high']
            : ['high']);
        this.signaling.send('publish_track', {
            kind,
            source,
            layers,
        });
    }
    async unpublishTrack(trackId) {
        this.signaling.send('unpublish_track', {
            track_id: trackId,
        });
        this.localTracks.delete(trackId);
    }
    async subscribe(trackId, options) {
        this.signaling.send('subscribe', {
            track_id: trackId,
            preferred_layer: options?.preferredLayer,
        });
    }
    async setPreferredLayer(trackId, layer) {
        this.signaling.send('layer_select', {
            track_id: trackId,
            layer,
        });
    }
    async sendData(payload, options) {
        this.signaling.send('data_message', {
            destination_peer_ids: options?.destinationPeerIds || [],
            payload,
            reliable: options?.reliable ?? true,
        });
    }
    async leave() {
        this.signaling.send('leave');
        this.roomId = null;
        this.peers.clear();
        this.tracks.clear();
        this.setState('connected');
        this.emit('roomLeft');
    }
    disconnect() {
        this.signaling.close();
        this.roomId = null;
        this.selfPeerId = null;
        this.peers.clear();
        this.tracks.clear();
        this.setState('disconnected');
        this.emit('disconnected');
    }
}
exports.AlvioRoom = AlvioRoom;
//# sourceMappingURL=room.js.map