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
| **Observability**| Prometheus `/metrics` & Health | `PLANNED` | Phase 11 | `/health` and `/ready` |
| **Scaling** | Node Placement & Drain Mode | `PLANNED` | Phase 12 | Authoritative room node |
| **Packaging** | Dockerfile & Docker Compose | `PLANNED` | Phase 13 | Self-hosted 1-command up |
| **Client SDK (Web)** | `@alviorelay/client` (TypeScript) | `PLANNED` | Phase 14 | Modern browsers (Chrome, Firefox, Safari, Edge) |
| **Client SDK (Desktop Windows)** | `alvio-desktop-windows` (C++ / Rust / WinUI) | `PLANNED` | Phase 14 | Native Windows `.exe`, low-latency audio/video capture |
| **Client SDK (Desktop macOS & Linux)** | `alvio-desktop-unix` (Swift / Metal / GTK / Tauri) | `PLANNED` | Phase 14 | Native macOS & Linux desktop apps |
| **Client SDK (Mobile Android)** | `alvio-android` (Kotlin / Android WebRTC) | `PLANNED` | Phase 14 | Android phones, tablets, smart TVs |
| **Client SDK (Mobile iOS)** | `alvio-ios` (Swift / iOS WebRTC) | `PLANNED` | Phase 14 | iPhone, iPad, Apple Silicon |
| **Client SDK (Cross-Platform)** | `alvio_flutter` & `@alviorelay/react-native` | `PLANNED` | Phase 14 | Multi-platform mobile/desktop hybrid apps |
| **Benchmarks** | Criterion Packet Routing Bench | `PLANNED` | Phase 15 | Sub-microsecond hot path |
