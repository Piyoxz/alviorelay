# alvio-egress

> **Virtual media tap and process-isolated FFmpeg recording supervisor for AlvioRelay WebRTC SFU.**

[![Documentation](https://docs.rs/alvio-egress/badge.svg)](https://docs.rs/alvio-egress)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-egress` enables recording and live transcoding without putting heavy encoding loads on the media hot path.

---

## Features

- **Non-Blocking Virtual Tap**: Subscribes to room audio and video tracks like a standard peer, receiving RTP without delaying other participants.
- **Isolated FFmpeg Supervisor**: FFmpeg executes in an isolated OS process. If an encoder segfaults or freezes, the SFU core remains 100% operational.
- **Automatic Multi-Format Egress**: Composite MP4 recordings or live HLS chunking written directly to local storage or cloud S3/MinIO.

---

## Installation

```toml
[dependencies]
alvio-egress = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
