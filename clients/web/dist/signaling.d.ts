import { EventHandler } from './types.js';
export declare class SignalingClient {
    private ws;
    private url;
    private listeners;
    private pingIntervalTimer;
    private isIntentionalClose;
    private reconnectAttempts;
    private maxReconnectAttempts;
    constructor();
    on(event: string, handler: EventHandler): void;
    off(event: string, handler: EventHandler): void;
    private emit;
    connect(url: string, token?: string, clientVersion?: string): Promise<void>;
    send(type: string, payload?: any): void;
    private startHeartbeat;
    private stopHeartbeat;
    close(): void;
}
//# sourceMappingURL=signaling.d.ts.map