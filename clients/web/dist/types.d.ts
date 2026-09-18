export type StreamKind = 'audio' | 'video';
export type StreamLayer = 'low' | 'medium' | 'high';
export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'in_room' | 'reconnecting' | 'failed';
export interface PeerInfo {
    id: string;
    name: string;
    metadata?: string;
}
export interface TrackInfo {
    id: string;
    peer_id: string;
    kind: StreamKind;
    source: string;
    layers: StreamLayer[];
}
export interface SignalEnvelope {
    version: number;
    id: string;
    type: string;
    payload?: any;
}
export interface PublishTrackOptions {
    source?: string;
    simulcast?: boolean;
    layers?: StreamLayer[];
}
export interface SubscribeOptions {
    preferredLayer?: StreamLayer;
}
export interface DataMessageOptions {
    destinationPeerIds?: string[];
    reliable?: boolean;
}
export type EventHandler<T = any> = (data: T) => void;
//# sourceMappingURL=types.d.ts.map