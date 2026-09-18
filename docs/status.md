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
| **Signaling** | WebSocket Handshake & Engine | `PLANNED` | Phase 2 | JSON v1 schema |
| **Signaling** | In-Memory Room & Peer Registry | `PLANNED` | Phase 2 | Lock-free DashMap |
| **WebRTC Core** | `AlvioTransport` abstraction | `PLANNED` | Phase 3 | str0m sans-I/O wrapper |
| **WebRTC Core** | ICE / DTLS-SRTP Handshake | `PLANNED` | Phase 3 | Peer-to-peer media path |
| **SFU** | RTP Packet Router (Hot Path) | `PLANNED` | Phase 4 | Zero-allocation Bytes |
| **SFU** | SSRC & Sequence Alignment | `PLANNED` | Phase 4 | Continuous numbering |
| **SFU** | NACK Ring Buffer & PLI / FIR | `PLANNED` | Phase 5 | Packet loss recovery |
| **SFU** | TWCC & Bandwidth Estimation | `PLANNED` | Phase 5 | Congestion control |
| **SFU** | Simulcast Layer Selection | `PLANNED` | Phase 6 | Low/Med/High switching |
| **Data** | WebRTC Data Channels | `PLANNED` | Phase 7 | Reliable & Unreliable |
| **Egress** | Virtual Recording Peer | `PLANNED` | Phase 8 | Non-blocking tap |
| **Egress** | FFmpeg Process Supervisor | `PLANNED` | Phase 8 | Isolated worker process |
| **Storage** | StorageBackend (Local & S3) | `PLANNED` | Phase 8 | Pluggable trait |
| **Ingress** | WHIP Endpoint | `PLANNED` | Phase 9 | OBS / WebRTC ingest |
| **Hooks** | Webhook Dispatcher & HMAC | `PLANNED` | Phase 10 | Retry & backoff |
| **Observability**| Prometheus `/metrics` & Health | `PLANNED` | Phase 11 | `/health` and `/ready` |
| **Scaling** | Node Placement & Drain Mode | `PLANNED` | Phase 12 | Authoritative room node |
| **Packaging** | Dockerfile & Docker Compose | `PLANNED` | Phase 13 | Self-hosted 1-command up |
| **Client SDK** | `@alviorelay/client` (TS) | `PLANNED` | Phase 14 | Browser library |
| **Benchmarks** | Criterion Packet Routing Bench | `PLANNED` | Phase 15 | Sub-microsecond hot path |
