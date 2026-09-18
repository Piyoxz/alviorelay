# alvio-ingress

> **WHIP (WebRTC-HTTP Ingestion Protocol) HTTP endpoint for OBS Studio, vMix, and FFmpeg in AlvioRelay.**

[![Documentation](https://docs.rs/alvio-ingress/badge.svg)](https://docs.rs/alvio-ingress)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-ingress` implements the IETF WHIP draft specification, allowing external hardware encoders and streaming software to push live audio/video directly into an AlvioRelay room using standard HTTP POST requests.

---

## Features

- **Standard WHIP Endpoint**: `POST /whip/{room_id}` accepts `application/sdp` and returns the SFU's SDP answer with HTTP `201 Created`.
- **OBS Studio 30+ Compatibility**: Native integration without any plugins required.
- **Resource Deletion**: `DELETE /whip/{room_id}/{session_id}` for graceful stream teardown.

---

## Installation

```toml
[dependencies]
alvio-ingress = "0.1.0"
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
