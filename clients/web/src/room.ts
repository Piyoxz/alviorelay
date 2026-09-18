import { SignalingClient } from './signaling.js';
import {
  ConnectionState,
  DataMessageOptions,
  EventHandler,
  PeerInfo,
  PublishTrackOptions,
  StreamKind,
  StreamLayer,
  SubscribeOptions,
  TrackInfo,
} from './types.js';

export class AlvioRoom {
  private signaling: SignalingClient;
  private state: ConnectionState = 'disconnected';
  private selfPeerId: string | null = null;
  private roomId: string | null = null;
  private peers: Map<string, PeerInfo> = new Map();
  private tracks: Map<string, TrackInfo> = new Map();
  private localTracks: Map<string, MediaStreamTrack> = new Map();
  private listeners: Map<string, Set<EventHandler>> = new Map();

  constructor() {
    this.signaling = new SignalingClient();
    this.setupSignalingListeners();
  }

  public getState(): ConnectionState {
    return this.state;
  }

  public getSelfPeerId(): string | null {
    return this.selfPeerId;
  }

  public getRoomId(): string | null {
    return this.roomId;
  }

  public getPeers(): PeerInfo[] {
    return Array.from(this.peers.values());
  }

  public getTracks(): TrackInfo[] {
    return Array.from(this.tracks.values());
  }

  public on(event: string, handler: EventHandler): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(handler);
  }

  public off(event: string, handler: EventHandler): void {
    const set = this.listeners.get(event);
    if (set) {
      set.delete(handler);
    }
  }

  private emit(event: string, data?: any): void {
    const handlers = this.listeners.get(event);
    if (handlers) {
      for (const h of handlers) {
        try {
          h(data);
        } catch (err) {
          console.error(`[AlvioRoom] Event error '${event}':`, err);
        }
      }
    }
  }

  private setState(newState: ConnectionState): void {
    this.state = newState;
    this.emit('stateChanged', newState);
  }

  private setupSignalingListeners(): void {
    this.signaling.on('ack', (payload: any) => {
      this.selfPeerId = payload.peer_id;
      this.setState('connected');
      this.emit('connected', { peerId: payload.peer_id, nodeId: payload.node_id });
    });

    this.signaling.on('room_joined', (payload: any) => {
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

    this.signaling.on('peer_joined', (payload: any) => {
      const peer = payload.peer;
      if (peer) {
        this.peers.set(peer.id, peer);
        this.emit('peerJoined', peer);
      }
    });

    this.signaling.on('peer_left', (payload: any) => {
      if (payload.peer_id) {
        this.peers.delete(payload.peer_id);
        this.emit('peerLeft', { peerId: payload.peer_id, reason: payload.reason });
      }
    });

    this.signaling.on('track_published', (payload: any) => {
      const track = payload.track;
      if (track) {
        this.tracks.set(track.id, track);
        this.emit('trackPublished', track);
      }
    });

    this.signaling.on('track_unpublished', (payload: any) => {
      if (payload.track_id) {
        this.tracks.delete(payload.track_id);
        this.emit('trackUnpublished', payload.track_id);
      }
    });

    this.signaling.on('data_received', (payload: any) => {
      this.emit('dataReceived', {
        sourcePeerId: payload.source_peer_id,
        payload: payload.payload,
      });
    });

    this.signaling.on('error', (payload: any) => {
      this.emit('error', payload);
    });

    this.signaling.on('close', () => {
      this.setState('disconnected');
      this.emit('disconnected');
    });

    this.signaling.on('reconnecting', (data: any) => {
      this.setState('reconnecting');
      this.emit('reconnecting', data);
    });
  }

  public async connect(url: string, token?: string): Promise<void> {
    this.setState('connecting');
    await this.signaling.connect(url, token);
  }

  public async join(roomId: string, peerName: string, metadata?: string): Promise<void> {
    this.signaling.send('join', {
      room_id: roomId,
      peer_name: peerName,
      metadata,
    });
  }

  public async publishTrack(
    track: MediaStreamTrack,
    options?: PublishTrackOptions,
  ): Promise<void> {
    const kind: StreamKind = track.kind === 'video' ? 'video' : 'audio';
    const source = options?.source || (kind === 'video' ? 'camera' : 'microphone');
    const layers: StreamLayer[] =
      options?.layers || (kind === 'video' && options?.simulcast
        ? ['low', 'medium', 'high']
        : ['high']);

    this.signaling.send('publish_track', {
      kind,
      source,
      layers,
    });
  }

  public async unpublishTrack(trackId: string): Promise<void> {
    this.signaling.send('unpublish_track', {
      track_id: trackId,
    });
    this.localTracks.delete(trackId);
  }

  public async subscribe(trackId: string, options?: SubscribeOptions): Promise<void> {
    this.signaling.send('subscribe', {
      track_id: trackId,
      preferred_layer: options?.preferredLayer,
    });
  }

  public async setPreferredLayer(trackId: string, layer: StreamLayer): Promise<void> {
    this.signaling.send('layer_select', {
      track_id: trackId,
      layer,
    });
  }

  public async sendData(payload: string, options?: DataMessageOptions): Promise<void> {
    this.signaling.send('data_message', {
      destination_peer_ids: options?.destinationPeerIds || [],
      payload,
      reliable: options?.reliable ?? true,
    });
  }

  public async leave(): Promise<void> {
    this.signaling.send('leave');
    this.roomId = null;
    this.peers.clear();
    this.tracks.clear();
    this.setState('connected');
    this.emit('roomLeft');
  }

  public disconnect(): void {
    this.signaling.close();
    this.roomId = null;
    this.selfPeerId = null;
    this.peers.clear();
    this.tracks.clear();
    this.setState('disconnected');
    this.emit('disconnected');
  }
}
