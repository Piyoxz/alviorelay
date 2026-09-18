# alvio-webrtc

> **WebRTC transport abstraction and Sans-I/O `str0m` wrapper for AlvioRelay WebRTC SFU.**

[![crates.io](https://img.shields.io/crates/v/alvio-webrtc.svg)](https://crates.io/crates/alvio-webrtc)
[![Documentation](https://docs.rs/alvio-webrtc/badge.svg)](https://docs.rs/alvio-webrtc)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-webrtc` encapsulates the core WebRTC state machine and media encryption layer behind a clean, memory-safe Rust interface. It handles ICE negotiation, DTLS handshake, SRTP encryption/decryption, and RTP header manipulation.

---

## Features

- **Sans-I/O Architecture**: Powered by `str0m`, strictly separating network socket I/O from protocol state logic.
- **Zero-Copy Packet Rewriting**: SSRC and sequence number normalization with zero heap reallocations.
- **Resilient Handshakes**: Automated ICE trickle candidate processing and DTLS certificate validation.

---

## Installation

```toml
[dependencies]
alvio-webrtc = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
