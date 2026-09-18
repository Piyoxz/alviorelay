# AlvioRelay

> **Rust-Native Self-Hosted Real-Time Media Infrastructure**

AlvioRelay is an independent, high-performance, self-hosted WebRTC Selective Forwarding Unit (SFU) and real-time media engine written in 100% pure Rust.

It is designed as **infrastructure**—not an end-user meeting application, not a clone, and not a wrapper around other media servers. Developers and operators use AlvioRelay to power video conferencing, audio rooms, voice calls, webinars, gaming audio, telemedicine, and AI-driven real-time audio/video streaming.

---

## 🧭 Documentation Portal

Kami memisahkan dokumentasi menjadi **dua jalur terpisah** agar Anda bisa langsung menemukan panduan yang sesuai dengan kebutuhan Anda:

| 🚀 [Jalur Pengguna & Operator](docs/user-guide.md) | 👨‍💻 [Jalur Developer & SDK](docs/README.md#👨💻-2-jalur-developer-software-engineer--integrator) |
| :--- | :--- |
| *Untuk administrator server, DevOps, atau siapapun yang ingin langsung menjalankan server:* | *Untuk software engineer yang ingin membuat aplikasi dan mengintegrasikan WebRTC:* |
| • **[Panduan Pengguna (User Guide)](docs/user-guide.md)** — Menjalankan server dalam 60 detik | • **[Signaling Protocol Spec (v1)](docs/protocol-spec.md)** — Skema JSON v1 WebSocket |
| • **[Panduan Deployment Produksi](docs/deployment.md)** — Docker, Systemd, Caddy & Nginx | • **[Web Client SDK Guide](docs/clients/web.md)** — TypeScript, React, Next.js |
| • **[Observability & Monitoring](docs/observability.md)** — Prometheus & Grafana visual | • **[SCTP Data Channels Guide](docs/data-channels.md)** — Chat, koordinat kursor, telemetry |
| • **[Troubleshooting & Tuning](docs/troubleshooting.md)** — Firewall, NAT, dan Linux sysctl | • **[WHIP Ingestion Guide](docs/whip-guide.md)** — Live stream dari OBS & FFmpeg |
| • **[Template Caddy & Nginx](deploy/)** — Konfigurasi SSL/TLS siap pakai | • **[Webhooks & Security Guide](docs/webhooks-guide.md)** — HMAC-SHA256 event delivery |

Lihat seluruh peta panduan di **[Dokumentasi Lengkap Hub (docs/README.md)](docs/README.md)**.

---

## ⚡ Quick Start

### Opsi 1: Jalankan dengan Docker Compose (Instan + Monitoring)
AlvioRelay, Prometheus, dan Dashboard Grafana siap jalan dalam satu perintah:

```bash
git clone https://github.com/Piyoxz/alviorelay.git
cd alviorelay

# Jalankan Relay (Port 7880), Prometheus (Port 9090), dan Grafana (Port 3001)
docker compose up -d

# Cek kesehatan server
curl http://localhost:7880/health
```
- **WebRTC Control & Signaling**: `http://localhost:7880`
- **Dashboard Grafana**: `http://localhost:3001` (user: `admin`, pass: `alviosecure`)

### Opsi 2: Kompilasi & Jalankan dari Source (Rust 1.80+)
```bash
# Validasi konfigurasi
cargo run -p alvio-relay -- check

# Nyalakan server
cargo run -p alvio-relay -- start
```

### Opsi 3: Buka Demo Web Interaktif
```bash
python -m http.server 3000 --directory clients/web/demo
```
Akses `http://localhost:3000` di browser Anda untuk mencoba webcam lokal, simulcast switcher, audio visualizer, chat interaktif via SCTP, dan monitor latensi real-time.

---

## 📱 Multi-Platform Client SDKs

AlvioRelay menyediakan fondasi SDK untuk seluruh platform utama:

| Platform | Bahasa / Framework | Panduan Integrasi |
| :--- | :--- | :--- |
| **Web** | TypeScript / JavaScript (`@alviorelay/client`) | [docs/clients/web.md](docs/clients/web.md) |
| **Desktop Windows** | C++ / Rust / WinUI / WASAPI (`alvio-client-core`) | [docs/clients/windows.md](docs/clients/windows.md) |
| **Desktop macOS & Linux** | Swift / Metal / GTK / PipeWire | [docs/clients/desktop-unix.md](docs/clients/desktop-unix.md) |
| **Mobile Android** | Kotlin / Android WebRTC / Camera2 | [docs/clients/android.md](docs/clients/android.md) |
| **Mobile iOS** | Swift / iOS WebRTC / CallKit | [docs/clients/ios.md](docs/clients/ios.md) |
| **Cross-Platform Flutter** | Dart / Flutter WebRTC (`alvio_flutter`) | [docs/clients/flutter.md](docs/clients/flutter.md) |
| **Cross-Platform React Native** | TypeScript TurboModule (`@alviorelay/react-native`) | [docs/clients/react-native.md](docs/clients/react-native.md) |

---

## 📦 Struktur Workspace Crate

```text
alviorelay/
├── Cargo.toml                    # Virtual Workspace root
├── alvio-relay.toml              # Konfigurasi server default
├── docker-compose.yml            # Stack Relay + Prometheus + Grafana
├── deploy/                       # Template deployment (Caddy, Nginx, Systemd, Grafana)
├── docs/                         # Pusat dokumentasi (Developer & Operator)
├── clients/                      # Client SDKs & Demo Web
│   ├── web/                      # TypeScript SDK & Runnable Browser Demo
│   ├── windows/                  # Native Windows integration
│   ├── android/                  # Android Kotlin integration
│   ├── ios/                      # iOS Swift integration
│   ├── flutter/                  # Flutter Dart integration
│   └── react-native/             # React Native integration
├── crates/
│   ├── alvio-core/               # Shared domain types, config loader, typed errors
│   ├── alvio-protocol/           # Versioned signaling protocol (JSON v1)
│   ├── alvio-webrtc/             # Transport abstraction over str0m (sans-I/O)
│   ├── alvio-sfu/                # RTP router hot path, NACK buffer, PLI/FIR, Simulcast
│   ├── alvio-signal/             # WebSocket room state machine & registry
│   ├── alvio-auth/               # Pluggable AuthProvider trait (NoAuth, JWT)
│   ├── alvio-storage/            # Pluggable StorageBackend (Local, S3)
│   ├── alvio-hooks/              # Webhook dispatcher dengan HMAC-SHA256
│   ├── alvio-observe/            # Tracing, Prometheus /metrics, dan health probes
│   ├── alvio-egress/             # Media recording & FFmpeg process supervisor
│   ├── alvio-ingress/            # WHIP HTTP ingestion endpoint
│   ├── alvio-cluster/            # Multi-node placement & drain mode
│   └── alvio-client-core/        # Cross-platform client engine & C-ABI FFI
└── services/
    └── relay/                    # Primary server binary ('alvio-relay')
```

---

## 📄 License

Dual-licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
