# alvio-relay

> **Primary server daemon and CLI for AlvioRelay WebRTC SFU Media Infrastructure.**

[![Documentation](https://docs.rs/alvio-relay/badge.svg)](https://docs.rs/alvio-relay)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-relay` is the primary executable binary of AlvioRelay. It brings together the WebSocket signaling server, WebRTC SFU media router, WHIP ingestion endpoints, Prometheus metrics, and storage subsystems into a single, highly-optimized standalone binary.

---

## CLI Usage

```bash
# Start the media server (uses alvio-relay.toml by default)
alvio-relay start

# Start with custom config path
alvio-relay start --config /etc/alvio/alvio-relay.toml

# Validate configuration file without starting
alvio-relay check --config alvio-relay.toml

# Dump active effective configuration
alvio-relay config

# Print version and target architecture
alvio-relay version
```

---

## Architecture Endpoints

| Path | Protocol | Purpose |
| :--- | :--- | :--- |
| `/ws` | WebSocket | Real-time signaling and peer negotiation |
| `/whip/{room_id}` | HTTP POST | WHIP live stream ingestion (OBS/FFmpeg) |
| `/metrics` | HTTP GET | Prometheus scraper metrics endpoint |
| `/health` | HTTP GET | Liveness probe (Kubernetes / Caddy) |
| `/ready` | HTTP GET | Readiness probe (Load balancers) |

---

## License

Dual-licensed under either Apache-2.0 or MIT.
