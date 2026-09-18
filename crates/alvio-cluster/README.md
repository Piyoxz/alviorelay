# alvio-cluster

> **Multi-node placement, routing, and graceful node draining for AlvioRelay WebRTC SFU.**

[![Documentation](https://docs.rs/alvio-cluster/badge.svg)](https://docs.rs/alvio-cluster)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-cluster` manages multi-node scaling across AlvioRelay instances, ensuring authoritative room ownership, consistent hashing, and zero-downtime maintenance through node draining.

---

## Features

- **Authoritative Room Placement**: Ensures all participants of a room connect to the same authoritative SFU node or edge proxy.
- **Graceful Drain Mode**: Marks nodes as draining so existing conferences finish naturally while new calls route to healthy nodes.
- **Node Heartbeats & Discovery**: Cluster health tracking with automatic stale node eviction.

---

## Installation

```toml
[dependencies]
alvio-cluster = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
