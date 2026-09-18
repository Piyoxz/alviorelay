# alvio-protocol

> **Versioned JSON signaling protocol specification and message envelopes for AlvioRelay WebRTC SFU.**

[![crates.io](https://img.shields.io/crates/v/alvio-protocol.svg)](https://crates.io/crates/alvio-protocol)
[![Documentation](https://docs.rs/alvio-protocol/badge.svg)](https://docs.rs/alvio-protocol)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-protocol` defines the serialization and deserialization envelopes for the WebSocket signaling contract connecting clients and the AlvioRelay server.

---

## Universal Envelope Schema (JSON v1)

```json
{
  "version": 1,
  "type": "string",
  "room_id": "string",
  "peer_id": "string",
  "payload": {}
}
```

---

## Supported Messages

- **Lifecycle**: `join`, `joined`, `leave`, `peer_joined`, `peer_left`
- **Negotiation**: `offer`, `answer`, `candidate`
- **Media Control**: `track_published`, `track_unpublished`, `layer_switch`
- **Heartbeat & System**: `ping`, `pong`, `error`

---

## Installation

```toml
[dependencies]
alvio-protocol = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
