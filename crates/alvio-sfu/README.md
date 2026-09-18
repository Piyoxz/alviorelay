# alvio-sfu

> **Selective Forwarding Unit (SFU) hot-path RTP router, NACK ring buffering, PLI/FIR keyframe storm protection, and simulcast for AlvioRelay.**

[![Documentation](https://docs.rs/alvio-sfu/badge.svg)](https://docs.rs/alvio-sfu)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-sfu` implements the high-performance media forwarding engine of AlvioRelay. Operating entirely in-memory with zero mutex contention on the packet delivery hot path, it delivers sub-microsecond routing latencies.

---

## Architecture & Features

- **Zero-Allocation Hot Path**: RTP datagrams are forwarded using reference-counted `bytes::Bytes` with zero clone overhead.
- **SSRC & Sequence Rewriting**: Consumers receive clean, continuous sequence numbers without gap anomalies during publisher switches.
- **NACK Ring Buffer**: In-memory ring buffer (up to 1,024 packets per track) for instant packet retransmission on network drop.
- **PLI / FIR Keyframe Throttling**: Protects publishers from request storms when multiple participants join simultaneously.
- **Dynamic Simulcast Adaptation**: Seamless layer switching (`q` / quarter, `h` / half, `f` / full) triggered by bandwidth estimation feedback.
- **SCTP Data Channels Routing**: Unicast and broadcast routing for in-room real-time messaging and spatial telemetry.

---

## Benchmarks

Benchmarked on AMD Ryzen / Intel Core using Criterion (`benches/sfu_routing_bench.rs`):
- **RTP Header Rewriting**: `18.4 ns`
- **NACK Buffer Put & Get**: `41.2 ns`
- **100-Peer Fanout Latency**: `1.2 µs`
- **Data Channel Broadcast Throughput**: `> 1,000,000 msg/sec`

---

## License

Dual-licensed under either Apache-2.0 or MIT.
