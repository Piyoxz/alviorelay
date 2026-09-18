import { ConnectionState, DataMessageOptions, EventHandler, PeerInfo, PublishTrackOptions, StreamLayer, SubscribeOptions, TrackInfo } from './types.js';
export declare class AlvioRoom {
    private signaling;
    private state;
    private selfPeerId;
    private roomId;
    private peers;
    private tracks;
    private localTracks;
    private listeners;
    constructor();
    getState(): ConnectionState;
    getSelfPeerId(): string | null;
    getRoomId(): string | null;
    getPeers(): PeerInfo[];
    getTracks(): TrackInfo[];
    on(event: string, handler: EventHandler): void;
    off(event: string, handler: EventHandler): void;
    private emit;
    private setState;
    private setupSignalingListeners;
    connect(url: string, token?: string): Promise<void>;
    join(roomId: string, peerName: string, metadata?: string): Promise<void>;
    publishTrack(track: MediaStreamTrack, options?: PublishTrackOptions): Promise<void>;
    unpublishTrack(trackId: string): Promise<void>;
    subscribe(trackId: string, options?: SubscribeOptions): Promise<void>;
    setPreferredLayer(trackId: string, layer: StreamLayer): Promise<void>;
    sendData(payload: string, options?: DataMessageOptions): Promise<void>;
    leave(): Promise<void>;
    disconnect(): void;
}
//# sourceMappingURL=room.d.ts.map