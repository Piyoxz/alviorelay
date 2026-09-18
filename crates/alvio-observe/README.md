# alvio-observe

> **Structured logging, Prometheus metrics collection, and health probes for AlvioRelay WebRTC SFU.**

[![crates.io](https://img.shields.io/crates/v/alvio-observe.svg)](https://crates.io/crates/alvio-observe)
[![Documentation](https://docs.rs/alvio-observe/badge.svg)](https://docs.rs/alvio-observe)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-observe` provides observability infrastructure for AlvioRelay, exposing Prometheus-compatible metrics (`/metrics`) and standardized health probes (`/health` and `/ready`).

---

## Features

- **Prometheus Metrics Exporter**: Tracks active rooms, connected peers, packet loss ratio, inbound/outbound bitrates, and keyframe/NACK requests.
- **Lock-Free Atomic Counters**: Zero mutex contention when updating high-frequency telemetry on the media hot path.
- **Kubernetes Health Probes**: Ready-to-use `/health` (liveness) and `/ready` (readiness) Axum route handlers.
- **Structured Tracing**: JSON and compact ANSI log formatting using `tracing-subscriber`.

---

## Installation

```toml
[dependencies]
alvio-observe = "0.1.0"
```

---

## Metric Catalog

| Metric Name | Type | Description |
| :--- | :--- | :--- |
| `alvio_active_rooms` | Gauge | Currently active conference rooms |
| `alvio_active_peers` | Gauge | Total WebRTC participants connected |
| `alvio_rtp_packets_in_total` | Counter | Cumulative inbound RTP media packets |
| `alvio_rtp_packets_out_total` | Counter | Cumulative outbound forwarded RTP packets |
| `alvio_packet_loss_ratio` | Gauge | Current packet loss percentage across sessions |

---

## License

Dual-licensed under either Apache-2.0 or MIT.
