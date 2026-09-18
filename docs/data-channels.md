# SCTP Data Channels Integration Guide

AlvioRelay includes full native support for WebRTC Data Channels powered by SCTP (Stream Control Transmission Protocol) encapsulated within DTLS.

Data Channels enable ultra-low latency bidirectional text and binary communication directly alongside audio and video tracks, eliminating the need for a secondary WebSocket connection for room messaging.

---

## 1. Reliability Modes

AlvioRelay supports both standard WebRTC reliability modes:

| Mode | Behavior | Ideal Use Cases |
| :--- | :--- | :--- |
| **`ReliableOrdered`** | Guaranteed packet delivery with automatic retransmissions and in-order processing. | Chat messages, hand raises, whiteboard drawing commands, poll voting, room state sync. |
| **`UnreliableLossy`** | Drops lost packets without retransmissions or head-of-line blocking. Lowest possible latency. | Shared mouse cursor positions, spatial audio 3D coordinates, telemetry, game controller state. |

---

## 2. Using Data Channels with `@alviorelay/client`

### 2.1 Sending Room Chat (Reliable Ordered)
```typescript
import { AlvioRoom } from "@alviorelay/client";

const room = new AlvioRoom({
  serverUrl: "wss://relay.example.com",
  roomId: "engineering-sync",
  peerId: "user-101",
  displayName: "Alex"
});

await room.connect();

// Send a broadcast message to all participants in the room
room.sendData(
  JSON.stringify({
    action: "chat_message",
    sender: "Alex",
    text: "Hello everyone! The deployment was successful.",
    timestamp: Date.now()
  }),
  { reliable: true }
);
```

### 2.2 Sending Real-time Cursor Coordinates (Unreliable Lossy)
```typescript
// Transmit mouse cursor coordinates at 60 FPS without buffering
window.addEventListener("pointermove", (event) => {
  const payload = new Float32Array([
    event.clientX / window.innerWidth,
    event.clientY / window.innerHeight
  ]);

  // Unreliable lossy transmission: if a packet drops, next frame supersedes it
  room.sendData(payload.buffer, { reliable: false });
});
```

### 2.3 Listening for Incoming Data Messages
```typescript
room.on("dataReceived", ({ senderPeerId, data }) => {
  if (typeof data === "string") {
    const message = JSON.parse(data);
    console.log(`[Chat] ${senderPeerId}: ${message.text}`);
  } else {
    // Binary payload (Float32Array)
    const coordinates = new Float32Array(data);
    updateRemoteCursor(senderPeerId, coordinates[0], coordinates[1]);
  }
});
```

---

## 3. Server Architecture & SFU Routing Hot-Path

In AlvioRelay, data channel messages bypass heavy JSON re-serialization whenever possible:
- Inbound SCTP datagrams are intercepted in `crates/alvio-sfu/src/router.rs`.
- Unicast or broadcast fan-out occurs entirely in-memory using zero-copy `bytes::Bytes`.
- Criterion benchmarks confirm throughput exceeding **1,000,000 messages/second** with under 1.5 microseconds latency per 100-peer fanout.
