# alvio-core

> **Core domain types, configuration loaders, and typed errors for AlvioRelay WebRTC SFU.**

[![crates.io](https://img.shields.io/crates/v/alvio-core.svg)](https://crates.io/crates/alvio-core)
[![Documentation](https://docs.rs/alvio-core/badge.svg)](https://docs.rs/alvio-core)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-core` is the foundational dependency of the [AlvioRelay](https://github.com/Piyoxz/alviorelay) workspace. It defines the core domain identifiers, strongly-typed errors, configuration parser (`alvio-relay.toml`), and shared utilities across all AlvioRelay subsystems.

---

## Features

- **Strict Type Safety**: Newtype wrappers for `RoomId`, `PeerId`, `TrackId`, `SessionId` preventing identifier mix-ups.
- **Hierarchical Configuration**: Load configuration from `alvio-relay.toml` with seamless environment variable overrides prefixed with `ALVIO_`.
- **Zero-Allocation Buffers**: Native integration with `bytes::Bytes` for media packet handling.
- **Robust Error Taxonomy**: Structured `AlvioError` enum powered by `thiserror`.

---

## Installation

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
alvio-core = "0.1.0"
```

---

## Quick Example

```rust
use alvio_core::{Config, RoomId, PeerId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Strongly typed domain IDs
    let room = RoomId::new("meeting-alpha");
    let peer = PeerId::new("user-42");

    println!("Connecting peer {} to room {}", peer, room);

    // Load server configuration
    let config = Config::load_from_path("alvio-relay.toml")?;
    println!("SFU listening on port: {}", config.server.http_port);

    Ok(())
}
```

---

## License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))
