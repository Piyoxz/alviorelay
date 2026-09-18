# AlvioRelay

> **Rust-Native Self-Hosted Real-Time Media Infrastructure**

AlvioRelay is an independent, high-performance, self-hosted WebRTC Selective Forwarding Unit (SFU) and real-time media engine written in 100% pure Rust.

It is designed as **infrastructure**—not an end-user meeting application, not a clone, and not a wrapper around other media servers. Developers use AlvioRelay to power video conferencing, audio rooms, voice calls, webinars, gaming audio, telemedicine, and AI-driven real-time audio/video streaming.

---

## Key Highlights

- 🦀 **Rust-First & Sans-I/O Architecture**: Powered by `str0m` wrapped behind our own `AlvioTransport` abstraction. Zero internal mutex contention on the media hot path.
- ⚡ **Zero Mandatory Database**: Room and peer states live 100% in-memory with lock-free, read-mostly concurrency (`DashMap` / `ArcSwap`). No PostgreSQL, MySQL, or Redis required to start.
- 🔌 **Pluggable Architecture**:
  - **Auth**: `NoAuth` (local dev), `JWT` (HMAC / Ed25519), `APIKey`, or custom providers.
  - **Storage**: `LocalFilesystem`, `S3Compatible` (AWS S3, MinIO, Cloudflare R2).
- 📦 **Minimal Footprint**: Single binary deployment (`alvio-relay`) or lightweight Docker container.
- 🔒 **Security by Default**: WSS signaling, DTLS-SRTP encryption, HMAC webhook verification, and zero credential leakage in logs.

---

## Quick Start

### 1. Build from Source
Ensure Rust 1.80+ is installed:

```bash
git clone https://github.com/Piyoxz/alviorelay.git
cd alviorelay

# Validate configuration
cargo run -p alvio-relay -- check

# Start the server
cargo run -p alvio-relay -- start
```

### 2. Configuration (`alvio-relay.toml`)
```toml
[server]
node_id = "alvio-node-01"
bind_address = "0.0.0.0"
http_port = 7880
log_level = "info"

[rtc]
udp_port = 7882
use_external_ip = false
ice_servers = [
    { urls = ["stun:stun.l.google.com:19302"] }
]

[auth]
provider = "no_auth"
```

All settings can be overridden via environment variables prefixed with `ALVIO_` (e.g. `ALVIO_HTTP_PORT=8080`, `ALVIO_RTC_UDP_PORT=7882`).

---

## Workspace Structure

```
alviorelay/
├── Cargo.toml                    # Virtual Workspace root
├── alvio-relay.toml              # Configuration
├── docs/                         # Architecture & ADRs
│   ├── adr/
│   ├── architecture.md
│   └── status.md
├── crates/
│   ├── alvio-core/               # Shared domain types, config loader, typed errors
│   ├── alvio-protocol/           # Versioned signaling protocol (JSON v1)
│   ├── alvio-webrtc/             # Transport abstraction over str0m
│   ├── alvio-sfu/                # Media routing hot path, SSRC remapping, NACK/PLI
│   ├── alvio-signal/             # WebSocket room state machine
│   ├── alvio-auth/               # Pluggable AuthProvider trait
│   ├── alvio-storage/            # Pluggable StorageBackend trait
│   ├── alvio-hooks/              # Webhook dispatcher with HMAC-SHA256
│   ├── alvio-observe/            # Tracing, Prometheus /metrics, and health probes
│   ├── alvio-egress/             # Media recording and FFmpeg supervisor
│   └── alvio-ingress/            # WHIP and external ingest adapters
└── services/
    └── relay/                    # Primary binary ('alvio-relay')
```

---

## Architecture Decision Records (ADRs)

Our design decisions are publicly recorded in `docs/adr/`:
- [ADR-0001: Modular Rust Workspace Architecture](docs/adr/ADR-0001-project-architecture.md)
- [ADR-0002: WebRTC Engine Selection & Sans-I/O Architecture](docs/adr/ADR-0002-webrtc-engine-selection.md)
- [ADR-0003: Versioned WebSocket Signaling Protocol](docs/adr/ADR-0003-signaling-protocol.md)
- [ADR-0004: Zero Mandatory Database & In-Memory State Model](docs/adr/ADR-0004-zero-mandatory-database.md)

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
