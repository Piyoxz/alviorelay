# alvio-signal

> **WebSocket signaling state machine and lock-free room registry for AlvioRelay WebRTC SFU.**

[![Documentation](https://docs.rs/alvio-signal/badge.svg)](https://docs.rs/alvio-signal)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-signal` handles participant lifecycles, SDP offer/answer exchanges, and trickle ICE candidate routing over Axum WebSockets.

---

## Features

- **Zero Mandatory Database**: Room and peer records reside in memory using lock-free, concurrent `DashMap` structures.
- **WebSocket Upgrade Engine**: Integrated with Axum 0.8 with per-message compression and client ping/pong heartbeats.
- **Broadcast Fan-Out**: Real-time distribution of `peer_joined`, `peer_left`, and `track_published` announcements.

---

## Installation

```toml
[dependencies]
alvio-signal = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
