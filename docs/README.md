# AlvioRelay Documentation Hub

Selamat datang di pusat dokumentasi resmi **AlvioRelay**. Dokumentasi ini dirancang ke dalam **dua jalur (track) terpisah** sesuai dengan peran dan kebutuhan Anda:

---

## 🧭 Pilih Jalur Anda

```
                    ┌───────────────────────────────┐
                    │      AlvioRelay Hub           │
                    └───────────────┬───────────────┘
                                    │
           ┌────────────────────────┴────────────────────────┐
           ▼                                                 ▼
┌───────────────────────────────┐         ┌───────────────────────────────┐
│   🚀 Jalur Pengguna & Admin   │         │     👨‍💻 Jalur Developer        │
│    (Orang yang Mau Memakai)   │         │   (Software Engineer / SDK)   │
├───────────────────────────────┤         ├───────────────────────────────┤
│ • Cara Jalankan Server (1 mnt)│         │ • Signaling Protocol JSON v1  │
│ • Panduan Konfigurasi TOML    │         │ • Client SDK Web / Mobile / DS│
│ • Docker Compose + Grafana    │         │ • SCTP Data Channels API      │
│ • Pasang Domain & SSL/TLS     │         │ • WHIP Ingest (OBS / FFmpeg)  │
│ • Firewall & Port Forwarding  │         │ • Webhooks & HMAC Verification│
│ • Panduan Troubleshooting     │         │ • Arsitektur Crate Internal   │
└───────────────────────────────┘         └───────────────────────────────┘
```

---

## 🚀 1. Jalur Pengguna & Operator (Self-Hoster)
*Ditujukan untuk administrator server, DevOps, atau siapapun yang ingin memasang dan menjalankan AlvioRelay tanpa perlu pusing dengan kode internal.*

| Panduan | Deskripsi Singkat |
| :--- | :--- |
| **[Panduan Pengguna (User Guide)](user-guide.md)** | **Mulai dari sini!** Penjelasan konsep dasar, cara menjalankan server dalam 60 detik, dan menghubungkan aplikasi pertama Anda. |
| **[Panduan Deployment Produksi](deployment.md)** | Panduan lengkap pasang di server/VPS: Docker Compose, Systemd service, Caddy/Nginx Reverse Proxy, dan pengaturan Firewall/NAT. |
| **[Pemantauan & Observability](observability.md)** | Cara memantau server dengan Prometheus dan dashboard visual Grafana bawaan, serta pengecekan health check `/health`. |
| **[Panduan Troubleshooting & Tuning](troubleshooting.md)** | Solusi praktis jika audio/video tidak muncul, port terblokir firewall, optimasi kernel Linux (`sysctl`), dan diagnosa packet loss. |

---

## 👨‍💻 2. Jalur Developer (Software Engineer & Integrator)
*Ditujukan untuk developer yang ingin mengintegrasikan video/audio call ke dalam aplikasi Web, Android, iOS, Windows, Mac, Flutter, atau React Native.*

### Integrasi Client SDK
| Platform | Bahasa / Framework | Panduan |
| :--- | :--- | :--- |
| **Web (Browser)** | TypeScript / JavaScript (React, Next.js, Vue) | **[Web Client Guide](clients/web.md)** |
| **Desktop Windows** | C++ / Rust / WinUI / WASAPI | **[Windows Desktop Guide](clients/windows.md)** |
| **Desktop macOS & Linux** | Swift / Metal / GTK / PipeWire | **[Desktop Unix Guide](clients/desktop-unix.md)** |
| **Mobile Android** | Kotlin / Android WebRTC / Camera2 | **[Android Guide](clients/android.md)** |
| **Mobile iOS** | Swift / iOS WebRTC / CallKit | **[iOS Guide](clients/ios.md)** |
| **Cross-Platform** | Flutter (Dart) | **[Flutter Guide](clients/flutter.md)** |
| **Cross-Platform** | React Native (iOS & Android) | **[React Native Guide](clients/react-native.md)** |

### Spesifikasi Protokol & Fitur Backend
| Dokumen | Deskripsi |
| :--- | :--- |
| **[Signaling Protocol Specification](protocol-spec.md)** | Spesifikasi lengkap skema JSON v1 WebSocket (`join`, `offer`, `answer`, `candidate`, dll). |
| **[SCTP Data Channels Guide](data-channels.md)** | Pengiriman data instan Reliable & Unreliable (chat, kursor bersama, telemetry, game sync). |
| **[WHIP Ingestion Guide](whip-guide.md)** | Cara streaming langsung dari OBS Studio atau FFmpeg ke room via endpoint HTTP POST `/whip/{room_id}`. |
| **[Webhooks & Security Guide](webhooks-guide.md)** | Menerima event server ke backend Anda dengan verifikasi tanda tangan HMAC-SHA256 (`Alvio-Signature`). |
| **[Arsitektur Internal AlvioRelay](architecture.md)** | Desain deep-dive Sans-I/O `str0m`, zero-allocation RTP router, dan isolasi proses rekaman. |
| **[Status Fitur & Roadmap](status.md)** | Matriks kelengkapan fitur dari Phase 0 hingga Phase 15. |

---

## 🧪 Demo Interaktif Web Client
AlvioRelay dilengkapi dengan aplikasi demo web yang siap dicoba langsung di browser Anda:
```bash
# Buka demo di folder clients/web/demo
python -m http.server 3000 --directory clients/web/demo
# Akses melalui browser: http://localhost:3000
```
Demo ini menampilkan webcam lokal, simulcast switcher, audio visualizer, chat interaktif via SCTP Data Channel, dan HUD status latensi real-time.
