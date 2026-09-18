# @alviorelay/client

> **Official TypeScript & JavaScript Web Client SDK for AlvioRelay WebRTC SFU.**

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`@alviorelay/client` provides an ergonomic, promise-based API for integrating real-time audio, video, screen sharing, and SCTP Data Channels into modern web applications (React, Next.js, Vue, Svelte, Angular, Vanilla JS).

---

## Features

- **One-Line Publishing**: Automatic camera and microphone capture via `publishTrack`.
- **Dynamic Simulcast**: Automatic 3-layer simulcast (`q` / `h` / `f`) with automatic layer down-switching on network congestion.
- **SCTP Data Channels**: Send instant broadcast chat messages or lossy 60 FPS cursor sync packets.
- **Auto-Reconnection**: Transparent exponential backoff reconnection when network switches (Wi-Fi to Cellular).
- **Zero Heavy Dependencies**: Lightweight, tree-shakeable, pure TypeScript.

---

## Installation

```bash
npm install @alviorelay/client
```

---

## Quick Example

```typescript
import { AlvioRoom } from "@alviorelay/client";

// 1. Initialize Room
const room = new AlvioRoom({
  serverUrl: "wss://relay.yourdomain.com",
  roomId: "conference-101",
  peerId: "user-alice",
  displayName: "Alice"
});

// 2. Listen for Events
room.on("trackSubscribed", ({ track, peerId }) => {
  const videoEl = document.createElement("video");
  videoEl.srcObject = new MediaStream([track]);
  videoEl.autoplay = true;
  videoEl.playsInline = true;
  document.getElementById("remote-grid")?.appendChild(videoEl);
});

room.on("dataReceived", ({ senderPeerId, data }) => {
  console.log(`Message from ${senderPeerId}:`, data);
});

// 3. Connect & Publish Local Media
await room.connect();
const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
stream.getTracks().forEach((track) => room.publishTrack(track));
```

---

## Interactive Demo

Try the interactive demo included in `demo/`:
```bash
python -m http.server 3000 --directory demo
# Open in browser: http://localhost:3000
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
