# AlvioRelay System Architecture

## 1. Overview
AlvioRelay is a Selective Forwarding Unit (SFU) designed from the ground up in Rust for zero-database self-hosting, maximum media throughput, and modular deployment.

## 2. Core Separation of Concerns

```
                  ┌───────────────────────────────┐
                  │    Signaling & Control Plane  │
                  │ (HTTP / WebSocket / Auth / WS)│
                  └───────────────┬───────────────┘
                                  │ Bounded Channel
                                  ▼
                  ┌───────────────────────────────┐
                  │       Media Hot Path Plane    │
                  │   (UDP Socket / Sans-I/O SFU) │
                  └───────────────┬───────────────┘
                                  │ Lockless Tap
                                  ▼
                  ┌───────────────────────────────┐
                  │    Egress & Recording Plane   │
                  │  (Process-Isolated Supervisor)│
                  └───────────────────────────────┘
```

### Plane Responsibilities:
1. **Control & Signaling Plane (`alvio-signal`, `alvio-auth`, `alvio-hooks`)**:
   - Manages WebSocket sessions, peer join/leave states, and SDP offer/answer exchanges.
   - Operates fully asynchronously on the Tokio threadpool.
   - Enforces authentication (`AuthProvider`) and dispatches outbound webhooks (`alvio-hooks`).
2. **Media Hot Path Plane (`alvio-sfu`, `alvio-webrtc`)**:
   - Manages incoming/outgoing UDP datagrams.
   - Powered by `str0m` Sans-I/O WebRTC state machine.
   - SSRC remapping, packet sequence numbering, and simulcast layer gating.
   - **Zero-allocation & lockless**: Never blocks on database queries, network HTTP calls, disk I/O, or mutex locks.
3. **Egress & Recording Plane (`alvio-egress`, `alvio-storage`)**:
   - Taps media streams out-of-band.
   - Supervises isolated `ffmpeg` worker processes.
   - Streams outputs to `StorageBackend` (Local disk or S3/MinIO).

## 3. Data Flow & Routing
- When Peer A sends video, packets arrive at the UDP socket loop.
- `AlvioTransport` decrypts SRTP into an RTP packet.
- SSRC lookup finds the registered `StreamSource`.
- The SFU iterates through subscribed `StreamConsumer`s:
  - Checks consumer layer preference (Low, Medium, High).
  - Translates SSRC to the consumer's agreed SSRC.
  - Rewrites sequence numbers to maintain a continuous stream without gaps.
- `AlvioTransport` encrypts the packet with the subscriber's SRTP keys and transmits it over UDP.

## 4. Multi-Platform Client Ecosystem

AlvioRelay is engineered as a **Universal Real-Time Media Server**. Any client capable of WebSocket JSON signaling and standard WebRTC (DTLS-SRTP, ICE, Opus/VP8/H.264/AV1) can participate:

| Target Platform | Technology Stack & SDK | Features & Capabilities |
| :--- | :--- | :--- |
| **Web Browser** | `@alviorelay/client` (TypeScript) | Native browser WebRTC W3C API, React / Vue / Svelte hooks, Screen share, Zero plugins required. |
| **Windows Desktop** | `alvio-desktop-windows` (C++ / Rust / WinUI / WPF) | Native `.exe`, WASAPI audio capture, Direct3D / DXGI screen capture, low-latency gaming & streaming. |
| **macOS & Linux Desktop** | `alvio-desktop-unix` (Swift Metal / GTK / Qt / Tauri) | CoreMedia / PipeWire capture, native tray apps, hardware acceleration. |
| **Mobile (Android)** | `alvio-android` (Kotlin / Android WebRTC) | Camera2 API, AudioRecord / AAudio, background call services, picture-in-picture. |
| **Mobile (iOS)** | `alvio-ios` (Swift / iOS WebRTC) | AVFoundation, CallKit integration, ReplayKit screen broadcasting. |
| **Cross-Platform Hybrid** | `alvio_flutter` & `@alviorelay/react-native` | Unified codebase for mobile and desktop apps with high-performance native bridges. |

