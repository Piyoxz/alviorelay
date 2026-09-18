# AlvioRelay

> **High-Performance, Rust-Native, Self-Hosted WebRTC Selective Forwarding Unit (SFU) & Real-Time Media Infrastructure.**

[![Crates.io](https://img.shields.io/crates/v/alvio-core.svg?label=crates.io%20%28alvio-core%29)](https://crates.io/crates/alvio-core)
[![Crates.io](https://img.shields.io/crates/v/alvio-webrtc.svg?label=crates.io%20%28alvio-webrtc%29)](https://crates.io/crates/alvio-webrtc)
[![Crates.io](https://img.shields.io/crates/v/alvio-protocol.svg?label=crates.io%20%28alvio-protocol%29)](https://crates.io/crates/alvio-protocol)
[![CI Status](https://github.com/Piyoxz/alviorelay/actions/workflows/ci.yml/badge.svg)](https://github.com/Piyoxz/alviorelay/actions/workflows/ci.yml)
[![Docker Image](https://img.shields.io/badge/docker-ready-2496ED.svg?logo=docker&logoColor=white)](docker-compose.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.80%2B-orange.svg?logo=rust&logoColor=white)](https://www.rust-lang.org)

AlvioRelay is an independent, high-performance, self-hosted WebRTC Selective Forwarding Unit (SFU) and real-time media engine written in 100% pure Rust.

It is designed as **pure infrastructure**—not a meeting clone or an opinionated web app. Developers and systems engineers use AlvioRelay to power video conferencing, audio rooms, voice calls, webinars, gaming audio, telemedicine, and AI-driven real-time audio/video streaming.

---

> [!NOTE]
> ### 🚀 Key Architecture Highlights
> - 🦀 **100% Pure Rust & Sans-I/O**: Media transport powered by `str0m` wrapped behind our clean `AlvioTransport` abstraction. Zero mutex locks on the media packet hot path.
> - ⚡ **Zero Mandatory Database**: All room registries, participant states, and routing tables live in memory using lock-free, concurrent `DashMap` structures. No PostgreSQL, MySQL, or Redis required to start.
> - 🎯 **Sub-Microsecond Latency**: RTP packet rewriting and 100-peer fan-out routing completes in **`1.2 µs`**.
> - 🎥 **WHIP Ingest**: Native WebRTC HTTP Ingestion Protocol endpoint (`/whip/{room_id}`) for OBS Studio, vMix, and FFmpeg without any third-party plugins.
> - 💬 **SCTP Data Channels**: Native reliable (chat, hand raises) and unreliable lossy (60 FPS shared cursors, spatial 3D audio, telemetry) messaging.
> - 📊 **Turnkey Observability**: Built-in Prometheus `/metrics` exporter and ready-to-import Grafana dashboard.

---

## 📑 Table of Contents

1. [Architectural Comparison](#-architectural-comparison)
2. [Quickstart in 60 Seconds (Docker)](#-1-quickstart-in-60-seconds-docker-compose)
3. [Building from Source (Cargo)](#-2-building-from-source-cargo)
4. [Interactive Web Demo](#-3-interactive-web-demo)
5. [Tutorial: Connecting with TypeScript SDK](#-4-tutorial-connecting-with-the-typescript-sdk)
6. [Tutorial: Live Streaming via OBS & FFmpeg (WHIP)](#-5-tutorial-live-streaming-via-obs--ffmpeg-whip)
7. [Tutorial: Real-Time Data Channels (Chat & Cursor)](#-6-tutorial-real-time-data-channels-chat--cursor)
8. [Tutorial: Webhooks & HMAC Verification](#-7-tutorial-webhooks--hmac-signature-verification)
9. [Tutorial: Production Deployment (SSL & Systemd)](#-8-tutorial-production-deployment-caddy-nginx--systemd)
10. [Tutorial: Linux Kernel UDP Tuning](#-9-tutorial-linux-kernel-udp-socket-tuning)
11. [Tutorial: Prometheus & Grafana Monitoring](#-10-tutorial-prometheus--grafana-monitoring)
12. [Multi-Platform Client SDKs](#-multi-platform-client-sdks)
13. [Configuration Reference (`alvio-relay.toml`)](#-configuration-reference-alvio-relaytoml)
14. [Criterion Micro-Benchmarks](#-criterion-micro-benchmarks)
15. [Workspace Structure](#-workspace-structure)
16. [License](#-license)

---

## ⚖️ Architectural Comparison

| Capability | AlvioRelay | LiveKit | Janus Gateway | Mediasoup | Pion |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Language** | **100% Pure Rust** | Go | C | C++ / Node.js | Go |
| **I/O Model** | **Sans-I/O (str0m)** | Socket I/O | Event-driven (C) | libuv Node worker | Socket I/O |
| **Mandatory Database** | **None (Zero DB)** | Redis required | None | None | None |
| **Hot Path Memory** | **Zero-Allocation Bytes**| Heap allocation | Raw C buffers | Node buffer bridge| Go GC overhead |
| **WHIP Ingest (OBS)**| **Native Built-in** | Ingress service | Plugin required | Wrapper required | Custom code |
| **Egress / Recording**| **Isolated FFmpeg** | Egress service | GStreamer plugin | Custom pipeline | Custom code |
| **Prometheus Exporter**| **Built-in `/metrics`**| Built-in | Plugin required | Custom agent | Custom |
| **Footprint (Single Binary)**| **~25 MB** | ~60 MB | ~40 MB + deps | Node.js + C++ addon| ~30 MB |

---

## ⚡ 1. Quickstart in 60 Seconds (Docker Compose)

The fastest way to experience AlvioRelay with instant visual telemetry:

```bash
# 1. Clone repository
git clone https://github.com/Piyoxz/alviorelay.git
cd alviorelay

# 2. Start AlvioRelay + Prometheus + Grafana in the background
docker compose up -d

# 3. Verify health probe
curl http://localhost:7880/health
# Response: {"status":"healthy"}
```

Services launched automatically:
- **AlvioRelay Control Plane & Signaling**: `http://localhost:7880`
- **AlvioRelay WebRTC UDP Media**: `udp://localhost:7882`
- **Prometheus Scraper**: `http://localhost:9090`
- **Grafana Live Telemetry Dashboard**: `http://localhost:3001` *(User: `admin`, Pass: `alviosecure`)*

---

## 🛠️ 2. Building from Source (Cargo)

Ensure **Rust 1.80+** is installed:

```bash
# Validate your configuration
cargo run -p alvio-relay -- check

# Start the primary relay server
cargo run -p alvio-relay -- start

# Dump effective active configuration
cargo run -p alvio-relay -- config
```

---

## 🌐 3. Interactive Web Demo

AlvioRelay includes a modern, zero-build interactive web showcase:

```bash
python -m http.server 3000 --directory clients/web/demo
```

Open `http://localhost:3000` in Google Chrome, Microsoft Edge, Safari, or Firefox:
- Live webcam and microphone preview with audio VU-meter.
- 3-tier simulcast layer switcher (`High`, `Medium`, `Low`).
- Real-time SCTP Data Channel chat drawer.
- Floating HUD displaying round-trip time (RTT), jitter, and packet stats.
- Modal SDK quickstart and code snippet generator.

---

## 💻 4. Tutorial: Connecting with the TypeScript SDK

Install the official client SDK:

```bash
npm install @alviorelay/client
```

### Complete Video Conference Example (React / Next.js / Vanilla JS)

```typescript
import { AlvioRoom } from "@alviorelay/client";

// 1. Initialize room connection
const room = new AlvioRoom({
  serverUrl: "wss://relay.yourdomain.com",
  roomId: "engineering-sync",
  peerId: "user-alice",
  displayName: "Alice Developer"
});

// 2. Handle remote participant video streams
room.on("trackSubscribed", ({ track, peerId, kind }) => {
  if (kind === "video") {
    const remoteVideo = document.createElement("video");
    remoteVideo.srcObject = new MediaStream([track]);
    remoteVideo.autoplay = true;
    remoteVideo.playsInline = true;
    document.getElementById("remote-video-container")?.appendChild(remoteVideo);
  }
});

// 3. Connect to room
await room.connect();

// 4. Capture and publish local webcam & microphone
const localStream = await navigator.mediaDevices.getUserMedia({
  video: { width: 1280, height: 720, frameRate: 30 },
  audio: { echoCancellation: true, noiseSuppression: true }
});

for (const track of localStream.getTracks()) {
  await room.publishTrack(track, {
    simulcast: track.kind === "video" // Enable 3-layer simulcast for video
  });
}

// 5. Screen Sharing
async function startScreenShare() {
  const displayStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
  const screenTrack = displayStream.getVideoTracks()[0];
  await room.publishTrack(screenTrack);
}
```

---

## 🎥 5. Tutorial: Live Streaming via OBS & FFmpeg (WHIP)

AlvioRelay includes native support for **WHIP (WebRTC-HTTP Ingestion Protocol)**.

### OBS Studio Setup (Version 30.0+)
1. Open **OBS Studio** -> **Settings** -> **Stream**.
2. Set **Service** to `WHIP`.
3. Set **Server** to:
   ```text
   https://relay.yourdomain.com/whip/my-live-stage
   ```
4. *(Optional)* Set **Bearer Token** if authentication is configured.
5. In **Settings** -> **Output**:
   - Encoder: **x264** or **NVIDIA NVENC H.264**
   - Rate Control: **CBR** (2500 Kbps - 6000 Kbps)
   - Keyframe Interval: **1s** or **2s**
   - Tune: **Zerolatency**
6. Click **Start Streaming**!

### Streaming via FFmpeg Command Line
```bash
ffmpeg -re -i input_video.mp4 \
  -c:v libx264 -preset ultrafast -tune zerolatency -b:v 3000k -g 30 \
  -c:a aac -b:a 128k \
  -f webrtc "http://localhost:7880/whip/my-live-stage"
```

---

## 💬 6. Tutorial: Real-Time Data Channels (Chat & Cursor)

Send structured data alongside media without setting up extra WebSocket connections:

```typescript
// A. Reliable Ordered: For In-Room Chat & State Sync
room.sendData(
  JSON.stringify({
    type: "chat",
    sender: "Alice",
    text: "The deployment is complete!",
    timestamp: Date.now()
  }),
  { reliable: true }
);

// B. Unreliable Lossy: For 60 FPS Shared Cursor Movement
window.addEventListener("pointermove", (event) => {
  const coords = new Float32Array([
    event.clientX / window.innerWidth,
    event.clientY / window.innerHeight
  ]);
  room.sendData(coords.buffer, { reliable: false });
});

// C. Receive incoming data messages
room.on("dataReceived", ({ senderPeerId, data }) => {
  if (typeof data === "string") {
    const msg = JSON.parse(data);
    renderChatMessage(senderPeerId, msg.text);
  } else {
    const [x, y] = new Float32Array(data);
    renderRemotePointer(senderPeerId, x, y);
  }
});
```

---

## 🔐 7. Tutorial: Webhooks & HMAC Signature Verification

Configure your backend URL in `alvio-relay.toml`:

```toml
[webhooks]
url = "https://api.yourcompany.com/webhooks/alvio"
secret = "super-secret-hmac-key"
```

### Verification in Node.js (Express)
```typescript
import crypto from "crypto";
import express from "express";

const app = express();
app.use(express.raw({ type: "application/json" })); // Preserve raw bytes

app.post("/webhooks/alvio", (req, res) => {
  const signature = req.headers["alvio-signature"] as string;
  const timestamp = req.headers["alvio-timestamp"] as string;

  // 1. Anti-Replay: Verify within 5 minutes
  if (Math.abs(Date.now() / 1000 - parseInt(timestamp, 10)) > 300) {
    return res.status(401).send("Timestamp expired");
  }

  // 2. Validate HMAC-SHA256 signature
  const hmac = crypto.createHmac("sha256", "super-secret-hmac-key");
  hmac.update(req.body);
  const expected = hmac.digest("hex");

  if (!crypto.timingSafeEqual(Buffer.from(signature, "hex"), Buffer.from(expected, "hex"))) {
    return res.status(403).send("Invalid signature");
  }

  const event = JSON.parse(req.body.toString("utf8"));
  console.log(`[Event Received] ${event.event_type} in room ${event.room_id}`);
  res.status(200).json({ success: true });
});
```

### Verification in Python (FastAPI)
```python
import hmac, hashlib, time
from fastapi import FastAPI, Request, HTTPException

app = FastAPI()
SECRET = b"super-secret-hmac-key"

@app.post("/webhooks/alvio")
async def handle_webhook(request: Request):
    sig = request.headers.get("alvio-signature")
    ts = request.headers.get("alvio-timestamp")
    
    if not sig or not ts or abs(time.time() - int(ts)) > 300:
        raise HTTPException(status_code=401, detail="Invalid headers or timestamp expired")
        
    raw_body = await request.body()
    computed = hmac.new(SECRET, raw_body, hashlib.sha256).hexdigest()
    
    if not hmac.compare_digest(sig, computed):
        raise HTTPException(status_code=403, detail="Invalid HMAC signature")
        
    event = await request.json()
    return {"status": "ok", "received_event": event["event_type"]}
```

---

## 🚀 8. Tutorial: Production Deployment (Caddy, Nginx & Systemd)

### Caddy Reverse Proxy (Automatic Free Let's Encrypt TLS)
Place into `/etc/caddy/Caddyfile`:

```caddyfile
relay.yourdomain.com {
    reverse_proxy 127.0.0.1:7880 {
        flush_interval -1
    }
}
```

### Linux Systemd Service Unit
Install our production unit file from [deploy/systemd/alvio-relay.service](deploy/systemd/alvio-relay.service):

```bash
sudo cp deploy/systemd/alvio-relay.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now alvio-relay
```

### Firewall Rules (UFW)
```bash
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 7880/tcp
sudo ufw allow 7882/udp
sudo ufw reload
```

---

## ⚙️ 9. Tutorial: Linux Kernel UDP Socket Tuning

For high-density production nodes handling hundreds of video streams, tune the kernel socket buffers in `/etc/sysctl.d/99-alvio.conf`:

```ini
# Maximum socket buffer sizes (32 MB)
net.core.rmem_max = 33554432
net.core.wmem_max = 33554432
net.core.rmem_default = 1048576
net.core.wmem_default = 1048576

# Network device backlog
net.core.netdev_max_backlog = 10000

# Concurrency file descriptors
fs.file-max = 2097152
```

Apply changes: `sudo sysctl --system`.

---

## 📊 10. Tutorial: Prometheus & Grafana Monitoring

AlvioRelay ships with a production-ready Grafana dashboard template at [deploy/grafana/dashboards/alvio-overview.json](deploy/grafana/dashboards/alvio-overview.json).

### Monitored Telemetry:
- **Active Rooms & Peers**: Real-time room scale and connected participants.
- **Inbound & Outbound Bandwidth**: Instantaneous ingress and egress fanout Mbps.
- **Packet Loss & NACKs**: Instant packet loss ratio and retransmission frequency.
- **Keyframe Rate**: Picture Loss Indication (PLI) request rate per second.
- **Data Channel Throughput**: Message frequency and data payload rates.

Access the pre-configured Grafana dashboard instantly at `http://localhost:3001` via `docker compose up -d`.

---

## 📱 Multi-Platform Client SDKs

AlvioRelay supports native applications across all major operating systems and mobile devices:

| Platform | Technology Stack | Documentation Guide |
| :--- | :--- | :--- |
| **Web Browsers** | TypeScript / ES Modules (`@alviorelay/client`) | [docs/clients/web.md](docs/clients/web.md) |
| **Windows Desktop** | C++ / Rust / WinUI / WASAPI (`alvio-client-core`) | [docs/clients/windows.md](docs/clients/windows.md) |
| **macOS & Linux** | Swift / Metal / GTK / PipeWire | [docs/clients/desktop-unix.md](docs/clients/desktop-unix.md) |
| **Android Mobile** | Kotlin / Camera2 API / AAudio | [docs/clients/android.md](docs/clients/android.md) |
| **iOS Mobile** | Swift / AVFoundation / CallKit | [docs/clients/ios.md](docs/clients/ios.md) |
| **Flutter Cross-Platform** | Dart / Flutter WebRTC (`alvio_flutter`) | [docs/clients/flutter.md](docs/clients/flutter.md) |
| **React Native** | TypeScript TurboModule (`@alviorelay/react-native`)| [docs/clients/react-native.md](docs/clients/react-native.md) |

---

## ⚙️ Configuration Reference (`alvio-relay.toml`)

```toml
[server]
node_id = "alvio-node-01"
bind_address = "0.0.0.0"
http_port = 7880
log_level = "info"

[rtc]
udp_port = 7882
use_external_ip = false
# external_ip = "203.0.113.45"
ice_servers = [
    { urls = ["stun:stun.l.google.com:19302"] }
]

[auth]
# "no_auth" for open access, or "jwt" with public key validation
provider = "no_auth"

[storage]
# "local" for local disk, or "s3" for AWS S3/MinIO
backend = "local"
local_path = "./recordings"

[webhooks]
# url = "https://api.example.com/webhooks/alvio"
# secret = "super-secret-hmac-key"
```

---

## 🔬 Criterion Micro-Benchmarks

Run benchmarks locally:
```bash
cargo bench --bench sfu_routing_bench
```

| Micro-Benchmark Routine | Time per Operation | Throughput | Notes |
| :--- | :--- | :--- | :--- |
| `rtp_header_parse_and_rewrite` | **18.4 ns** | ~54.3M packets/sec | In-place byte manipulation |
| `nack_buffer_put_and_get` | **41.2 ns** | ~24.2M packets/sec | Lock-free ring buffer |
| `sfu_routing_fanout_10_peers` | **124.5 ns** | ~8.0M frames/sec | Zero-copy byte cloning |
| `sfu_routing_fanout_100_peers` | **1.21 µs** | ~826K frames/sec | Multi-subscriber distribution |
| `data_router_broadcast_100_peers`| **1.14 µs** | ~877K msg/sec | Sub-microsecond SCTP routing |

---

## 📦 Workspace Structure

```text
alviorelay/
├── Cargo.toml                    # Virtual Workspace definition
├── alvio-relay.toml              # Server configuration
├── docker-compose.yml            # AlvioRelay + Prometheus + Grafana stack
├── deploy/                       # Caddy, Nginx, Systemd, Grafana provisioning
├── docs/                         # Dual documentation portal (Developer & Operator)
├── clients/                      # Multi-platform Client SDKs & Web Demo
├── crates/
│   ├── alvio-core/               # Shared domain types & config parser
│   ├── alvio-observe/            # Prometheus metrics & health probes
│   ├── alvio-storage/            # Storage backend abstraction (Local & S3)
│   ├── alvio-webrtc/             # Sans-I/O WebRTC str0m transport wrapper
│   ├── alvio-protocol/           # JSON v1 signaling protocol specification
│   ├── alvio-sfu/                # RTP routing hot path, NACK buffer, Simulcast
│   ├── alvio-signal/             # WebSocket room state machine & registry
│   ├── alvio-egress/             # Process-isolated FFmpeg recording
│   ├── alvio-ingress/            # WHIP HTTP live stream ingestion endpoint
│   ├── alvio-hooks/              # HMAC-SHA256 asynchronous webhook engine
│   ├── alvio-cluster/            # Multi-node placement & graceful node draining
│   └── alvio-client-core/        # Cross-platform client engine & C-ABI FFI
└── services/
    └── relay/                    # Primary executable binary daemon ('alvio-relay')
```

---

## 📄 License

Dual-licensed under either of:
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE))
- **MIT License** ([LICENSE-MIT](LICENSE-MIT))

at your option.
