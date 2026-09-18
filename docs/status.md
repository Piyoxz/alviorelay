# AlvioRelay Feature Status

Classification:
- `IMPLEMENTED`: Fully completed, tested, and documented.
- `PARTIALLY_IMPLEMENTED`: Functional core in place, advanced edge-cases pending.
- `EXPERIMENTAL`: Work in progress, breaking changes expected.
- `PLANNED`: Designed in architecture plan, pending execution phase.

---

| Subsystem | Feature | Status | Target Phase | Notes |
| :--- | :--- | :--- | :--- | :--- |
| **Foundation** | Rust Virtual Workspace & ADRs | `IMPLEMENTED` | Phase 0 | Clean modular structure |
| **Foundation** | Config Loader (TOML & Env) | `IMPLEMENTED` | Phase 1 | `alvio-core/src/config.rs` |
| **Foundation** | Core Domain Types & Errors | `IMPLEMENTED` | Phase 1 | `alvio-core/src/types.rs` |
| **Foundation** | Structured Logging & Signals | `IMPLEMENTED` | Phase 1 | `alvio-observe` |
| **CLI** | `alvio-relay` primary binary | `IMPLEMENTED` | Phase 1 | start, check, config, version |
| **Signaling** | WebSocket Handshake & Engine | `IMPLEMENTED` | Phase 2 | JSON v1 schema & Axum WS |
| **Signaling** | In-Memory Room & Peer Registry | `IMPLEMENTED` | Phase 2 | Lock-free DashMap & pub/sub |
| **WebRTC Core** | `AlvioTransport` abstraction | `IMPLEMENTED` | Phase 3 | str0m sans-I/O wrapper |
| **WebRTC Core** | ICE / DTLS-SRTP Handshake | `IMPLEMENTED` | Phase 3 | Peer-to-peer media path |
| **SFU** | RTP Packet Router (Hot Path) | `IMPLEMENTED` | Phase 4 | Zero-allocation Bytes |
| **SFU** | SSRC & Sequence Alignment | `IMPLEMENTED` | Phase 4 | Continuous numbering |
| **SFU** | NACK Ring Buffer & PLI / FIR | `IMPLEMENTED` | Phase 5 | Packet loss recovery |
| **SFU** | TWCC & Bandwidth Estimation | `IMPLEMENTED` | Phase 5 | Congestion control |
| **SFU** | Simulcast Layer Selection | `IMPLEMENTED` | Phase 6 | Low/Med/High switching |
| **Data** | WebRTC Data Channels | `IMPLEMENTED` | Phase 7 | Reliable & Unreliable |
| **Egress** | Virtual Recording Peer | `IMPLEMENTED` | Phase 8 | Non-blocking tap |
| **Egress** | FFmpeg Process Supervisor | `IMPLEMENTED` | Phase 8 | Isolated worker process |
| **Storage** | StorageBackend (Local & S3) | `IMPLEMENTED` | Phase 8 | Pluggable trait |
| **Ingress** | WHIP Endpoint | `IMPLEMENTED` | Phase 9 | OBS / WebRTC ingest |
| **Hooks** | Webhook Dispatcher & HMAC | `IMPLEMENTED` | Phase 10 | Retry & backoff |
| **Observability**| Prometheus `/metrics` & Health | `IMPLEMENTED` | Phase 11 | `/health` and `/ready` |
| **Scaling** | Node Placement & Drain Mode | `IMPLEMENTED` | Phase 12 | Authoritative room node |
| **Packaging** | Dockerfile & Docker Compose | `IMPLEMENTED` | Phase 13 | Self-hosted 1-command up |
| **Client SDK (Web)** | `@alviorelay/client` (TypeScript) | `IMPLEMENTED` | Phase 14 | Modern browsers (Chrome, Firefox, Safari, Edge) |
| **Client SDK (Desktop Windows)** | `alvio-desktop-windows` (C++ / Rust / WinUI) | `IMPLEMENTED` | Phase 14 | Native Windows `.exe`, low-latency audio/video capture |
| **Client SDK (Desktop macOS & Linux)** | `alvio-desktop-unix` (Swift / Metal / GTK / Tauri) | `IMPLEMENTED` | Phase 14 | Native macOS & Linux desktop apps |
| **Client SDK (Mobile Android)** | `alvio-android` (Kotlin / Android WebRTC) | `IMPLEMENTED` | Phase 14 | Android phones, tablets, smart TVs |
| **Client SDK (Mobile iOS)** | `alvio-ios` (Swift / iOS WebRTC) | `IMPLEMENTED` | Phase 14 | iPhone, iPad, Apple Silicon |
| **Client SDK (Cross-Platform)** | `alvio_flutter` & `@alviorelay/react-native` | `IMPLEMENTED` | Phase 14 | Multi-platform mobile/desktop hybrid apps |
| **Benchmarks** | Criterion Packet Routing Bench | `IMPLEMENTED` | Phase 15 | Sub-microsecond hot path |

---

## Full Documentation Pages Roadmap & Specification

Rencana komprehensif halaman dokumentasi resmi AlvioRelay untuk memandu developer, software engineer, dan system administrator dalam integrasi dan operasional:

| Halaman Dokumentasi | Path Target | Target Pembaca & Cakupan Materi | Status |
| :--- | :--- | :--- | :--- |
| **1. Architecture Deep-Dive** | `docs/architecture.md` | Core developers. Pemisahan bidang kontrol (Control Plane), Hot-Path SFU Sans-I/O `str0m`, zero-allocation routing, dan process-isolated recording. | `LIVE` |
| **2. Signaling Protocol Spec** | `docs/protocol-spec.md` | Client SDK developers. Schema JSON v1 message envelopes (`join`, `offer`, `answer`, `candidate`, `track_published`, `peer_left`), heartbeats, dan reconnection backoff. | `LIVE` |
| **3. SCTP Data Channels Guide** | `docs/data-channels.md` | Developers. Pembuatan kanal data `Reliable` (chat, izin) vs `UnreliableLossy` (koordinat kursor, telemetry, game state) berlatensi ultra-rendah. | `LIVE` |
| **4. WHIP Ingestion Guide** | `docs/whip-guide.md` | Streamers & Broadcasters. Panduan konfigurasi OBS Studio, vMix, FFmpeg, dan WebRTC camera encoders langsung ke endpoint HTTP POST `/whip/{room_id}`. | `LIVE` |
| **5. Webhook & Security Guide** | `docs/webhooks-guide.md` | Backend developers. Verifikasi tanda tangan HMAC-SHA256 (`Alvio-Signature`), perlindungan replay attack (`Alvio-Timestamp`), dan katalog event lengkap. | `LIVE` |
| **6. Client SDK (Web)** | `docs/clients/web.md` | Frontend developers. `@alviorelay/client` untuk React, Next.js, Vue, Svelte: audio/video publish, subscribe, layer switching simulcast, dan screen share. | `LIVE` |
| **7. Client SDK (Desktop Windows)** | `docs/clients/windows.md` | Desktop developers. Native Windows `.exe` (C++ / Rust / WinUI / WPF), WASAPI low-latency audio capture, DirectX / DXGI screen capture. | `LIVE` |
| **8. Client SDK (macOS & Linux)** | `docs/clients/desktop-unix.md` | Desktop developers. Native macOS (Swift / Metal / CoreMedia) dan Linux (GTK / Tauri / PipeWire audio capture). | `LIVE` |
| **9. Client SDK (Mobile Android)** | `docs/clients/android.md` | Mobile developers. Kotlin SDK, integrasi Camera2 API, AudioRecord / AAudio, background service calling, dan Picture-in-Picture (PiP). | `LIVE` |
| **10. Client SDK (Mobile iOS)** | `docs/clients/ios.md` | Mobile developers. Swift SDK, AVFoundation capture, CallKit system incoming call UI, dan ReplayKit in-app screen broadcast. | `LIVE` |
| **11. Client SDK (Cross-Platform Flutter)** | `docs/clients/flutter.md` | Hybrid developers. `alvio_flutter` (Flutter Dart API) multi-platform mobile/desktop renderers. | `LIVE` |
| **12. Client SDK (Cross-Platform React Native)** | `docs/clients/react-native.md` | Hybrid developers. `@alviorelay/react-native` TurboModule bridge untuk iOS dan Android. | `LIVE` |
| **13. Observability & Dashboards** | `docs/observability.md` | DevOps & SRE. Prometheus `/metrics` scraping, Grafana dashboard JSON template, alerting rules, dan probe `/health`, `/ready`. | `LIVE` |
| **14. Production Deployment** | `docs/deployment.md` | DevOps. 1-command Docker Compose, bare-metal systemd setup, reverse proxy Nginx/Caddy TLS termination, dan multi-node clustering. | `LIVE` |
| **15. Tuning & Troubleshooting** | `docs/troubleshooting.md` | SRE. Linux sysctl UDP buffer tuning (`rmem_max`, `wmem_max`), debugging packet loss, diagnosa NACK storm, dan profiling latency. | `LIVE` |

