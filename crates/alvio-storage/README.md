# alvio-storage

> **Pluggable storage backend abstraction (Local Filesystem & S3-compatible) for AlvioRelay WebRTC SFU.**

[![crates.io](https://img.shields.io/crates/v/alvio-storage.svg)](https://crates.io/crates/alvio-storage)
[![Documentation](https://docs.rs/alvio-storage/badge.svg)](https://docs.rs/alvio-storage)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-storage` defines the asynchronous `StorageBackend` trait powering egress recordings and persistent media assets in AlvioRelay.

---

## Features

- **Asynchronous Storage Trait**: Universal `put`, `get`, `exists`, `delete`, and streaming upload operations.
- **Local Storage Engine**: High-throughput file writing with parent directory auto-creation and atomic writes.
- **S3-Compatible Cloud Storage**: Ready for AWS S3, MinIO, Cloudflare R2, Wasabi, and DigitalOcean Spaces.

---

## Installation

```toml
[dependencies]
alvio-storage = "0.1.0"
```

---

## Example Usage

```rust
use alvio_storage::{StorageBackend, LocalStorage};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = LocalStorage::new("./recordings");

    let data = Bytes::from_static(b"sample video bytes");
    storage.put("room-alpha/meeting.mp4", data).await?;

    assert!(storage.exists("room-alpha/meeting.mp4").await?);
    Ok(())
}
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
