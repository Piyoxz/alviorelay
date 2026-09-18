# alvio-hooks

> **Asynchronous webhook dispatcher with HMAC-SHA256 signature verification for AlvioRelay WebRTC SFU.**

[![Documentation](https://docs.rs/alvio-hooks/badge.svg)](https://docs.rs/alvio-hooks)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-hooks` dispatches asynchronous HTTP POST notifications to backend application servers when room, peer, and egress recording events occur.

---

## Features

- **HMAC-SHA256 Signatures**: Every request includes an `Alvio-Signature` header computed over the raw body using a shared secret.
- **Replay Attack Defense**: Includes an `Alvio-Timestamp` header with a 5-minute validity window.
- **Resilient Delivery**: Automatic retry with exponential backoff for transient server failures.

---

## Event Catalog

- `room_created`, `room_destroyed`
- `peer_joined`, `peer_left`
- `track_published`, `track_unpublished`
- `recording_completed`

---

## Installation

```toml
[dependencies]
alvio-hooks = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
