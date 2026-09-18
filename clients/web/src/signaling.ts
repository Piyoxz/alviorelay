import { EventHandler, SignalEnvelope } from './types.js';

export class SignalingClient {
  private ws: WebSocket | null = null;
  private url: string = '';
  private listeners: Map<string, Set<EventHandler>> = new Map();
  private pingIntervalTimer: any = null;
  private isIntentionalClose: boolean = false;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 5;

  constructor() {}

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
          console.error(`[SignalingClient] Handler error on '${event}':`, err);
        }
      }
    }
  }

  public async connect(url: string, token?: string, clientVersion: string = 'web-0.1.0'): Promise<void> {
    this.url = url;
    this.isIntentionalClose = false;

    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(url);
      } catch (err) {
        return reject(err);
      }

      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.startHeartbeat();

        // Send initial Connect handshake
        this.send('connect', {
          token,
          client_version: clientVersion,
        });

        this.emit('open');
        resolve();
      };

      this.ws.onmessage = (event) => {
        try {
          const envelope: SignalEnvelope = JSON.parse(event.data);
          this.emit('message', envelope);
          if (envelope.type) {
            this.emit(envelope.type, envelope.payload);
          }
        } catch (err) {
          console.warn('[SignalingClient] Failed to parse signaling message:', err);
        }
      };

      this.ws.onerror = (err) => {
        this.emit('error', err);
      };

      this.ws.onclose = () => {
        this.stopHeartbeat();
        this.emit('close');

        if (!this.isIntentionalClose && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          const backoff = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 10000);
          this.emit('reconnecting', { attempt: this.reconnectAttempts, delayMs: backoff });
          setTimeout(() => {
            if (!this.isIntentionalClose) {
              this.connect(this.url).catch(() => {});
            }
          }, backoff);
        }
      };
    });
  }

  public send(type: string, payload?: any): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      throw new Error('[SignalingClient] WebSocket is not connected');
    }

    const envelope: SignalEnvelope = {
      version: 1,
      id: `msg_${Math.random().toString(36).substring(2, 11)}`,
      type,
      payload,
    };

    this.ws.send(JSON.stringify(envelope));
  }

  private startHeartbeat(): void {
    this.stopHeartbeat();
    this.pingIntervalTimer = setInterval(() => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.send('ping');
      }
    }, 15000);
  }

  private stopHeartbeat(): void {
    if (this.pingIntervalTimer) {
      clearInterval(this.pingIntervalTimer);
      this.pingIntervalTimer = null;
    }
  }

  public close(): void {
    this.isIntentionalClose = true;
    this.stopHeartbeat();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}
