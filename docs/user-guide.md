# Panduan Pengguna & Operator AlvioRelay (User Guide)

> **Untuk siapa panduan ini?**  
> Panduan ini ditulis khusus untuk Anda yang **ingin langsung menggunakan atau meng-hosting AlvioRelay sendiri** di server, VPS, atau komputer lokal Anda, **tanpa perlu mengerti bahasa pemrograman Rust**.

---

## 💡 1. Apa itu AlvioRelay Secara Sederhana?

Bayangkan AlvioRelay seperti **stasiun pemancar audio-video pribadi** (seperti mesin di balik Zoom, Google Meet, atau Discord voice channel), tetapi:
1. **Milik Anda Sendiri (Self-Hosted)**: Data suara dan video Anda tidak dikirim ke server pihak ketiga.
2. **Tanpa Database yang Ribet**: Anda **TIDAK** perlu menginstal PostgreSQL, MySQL, ataupun Redis. AlvioRelay berjalan langsung di memori RAM, sehingga sangat ringan dan hemat sumber daya.
3. **Sangat Cepat**: Menggunakan teknologi WebRTC standar industri dengan latensi sub-detik (di bawah 100 milidetik).

---

## ⚡ 2. Cara Menjalankan dalam 60 Detik

### Cara A: Menggunakan Docker (Paling Disarankan & Termudah)
Jika komputer atau server Anda sudah memiliki Docker:

```bash
# 1. Masuk ke direktori alviorelay
cd alviorelay

# 2. Jalankan AlvioRelay + Prometheus untuk pemantauan
docker compose up -d

# 3. Cek apakah server sudah aktif
curl http://localhost:7880/health
# Jika respon: {"status":"healthy"} artinya server Anda sudah BERJALAN SEMPURNA!
```

### Cara B: Menggunakan Binary Langsung (Tanpa Docker)
Jika Anda memiliki binary file `alvio-relay` (atau mengkompilasi dengan Rust):

```bash
# 1. Cek apakah konfigurasi valid
./alvio-relay check

# 2. Jalankan server
./alvio-relay start
```
Server akan menyala dan siap menerima koneksi pada port `7880`.

---

## ⚙️ 3. Penjelasan File Konfigurasi (`alvio-relay.toml`)

File `alvio-relay.toml` adalah pusat pengaturan server Anda. Semua pengaturannya menggunakan bahasa yang sangat jelas:

```toml
[server]
# Nama unik node server Anda (bebas diubah)
node_id = "alvio-node-01"

# Alamat IP server mendengarkan (0.0.0.0 berarti semua koneksi masuk diperbolehkan)
bind_address = "0.0.0.0"

# Port untuk signaling WebSocket, Web Demo, dan API
http_port = 7880

# Tingkat rincian catatan log: "info", "warn", "error", atau "debug"
log_level = "info"

[rtc]
# Port UDP untuk streaming audio dan video WebRTC
udp_port = 7882

# JIKA SERVER DI VPS / CLOUD: Ganti ke true agar browser luar bisa terhubung
use_external_ip = false
# external_ip = "203.0.113.45" # Masukkan IP Publik VPS Anda jika use_external_ip = true

# STUN Server gratis dari Google untuk membantu koneksi menembus router NAT
ice_servers = [
    { urls = ["stun:stun.l.google.com:19302"] }
]

[auth]
# Untuk pemula/uji coba: "no_auth" (siapa saja bebas masuk room)
# Untuk produksi: "jwt" (hanya yang memiliki token rahasia yang boleh masuk)
provider = "no_auth"
```

> [!TIP]
> **Penting untuk pengguna VPS / Cloud (DigitalOcean, AWS, GCP, Linode)**:
> Jika Anda menginstall di VPS, pastikan untuk membuka:
> 1. **Port TCP 7880** (Signaling & HTTP)
> 2. **Port UDP 7882** (Audio & Video WebRTC)
> Jika port UDP tidak dibuka, browser akan terlihat "Connecting..." tetapi video tidak akan pernah muncul.

---

## 🌐 4. Cara Mencoba Menggunakan Web Client Demo

AlvioRelay sudah dilengkapi dengan aplikasi halaman web demo interaktif:

1. **Jalankan server demo web di komputer Anda:**
   ```bash
   python -m http.server 3000 --directory clients/web/demo
   ```
2. **Buka di browser:**
   Kunjungi `http://localhost:3000` di Google Chrome, Mozilla Firefox, Microsoft Edge, atau Safari.
3. **Mulai Video Call:**
   - Masukkan **Room ID** (contoh: `ruang-rapat-1`) dan **Nama Anda** (contoh: `Budi`).
   - Klik **"Connect & Join"**.
   - Berikan izin akses Kamera dan Mikrofon saat browser memintanya.
   - Buka tab baru di browser Anda atau komputer lain di jaringan yang sama, masukkan Room ID yang sama (`ruang-rapat-1`) dengan nama berbeda (contoh: `Siti`).
   - **Selamat! Anda sudah berhasil membuat konferensi video WebRTC pribadi!**

---

## 🔒 5. Mengapa Anda Membutuhkan Domain & SSL (HTTPS)?

Bila Anda ingin menyebarkan AlvioRelay ke internet agar teman atau pelanggan bisa mengaksesnya dari luar:
- **Aturan Wajib Browser**: Google Chrome, Safari, dan Firefox **menolak** memberikan izin akses mikrofon dan kamera jika website tidak memakai HTTPS (`https://` atau `wss://`), kecuali di `localhost`.
- **Solusi Paling Mudah**: Gunakan **Caddy** sebagai reverse proxy karena Caddy otomatis mengurus dan memperbarui sertifikat SSL gratis dari Let's Encrypt tanpa konfigurasi rumit.
- Contoh konfigurasi Caddy lengkap tersedia di panduan **[Production Deployment](deployment.md)**.

---

## ❓ 6. FAQ (Pertanyaan yang Sering Muncul)

### Q: Berapa banyak pengguna yang bisa ditampung AlvioRelay?
**J:** Pada server berspesifikasi 4 vCPU dan 4 GB RAM, AlvioRelay yang ditulis dalam pure Rust mampu merutekan puluhan hingga ratusan stream video simultan berlatensi ultra-rendah dengan konsumsi memori di bawah 100 MB.

### Q: Kenapa kamera saya menyala tapi teman saya tidak bisa melihat videonya?
**J:** 99% masalah ini disebabkan oleh firewall server yang memblokir port UDP. Pastikan port **UDP 7882** sudah diizinkan di firewall VPS Anda (`sudo ufw allow 7882/udp`).

### Q: Apakah video dan audio saya direkam di server?
**J:** Secara default, AlvioRelay **hanya merutekan paket data di memori** dan TIDAK menyimpan rekaman apa pun. Jika Anda ingin merekam, aktifkan modul `alvio-egress` di konfigurasi Anda.

### Q: Bagaimana cara melihat berapa banyak orang yang sedang online?
**J:** Buka alamat `http://alamat-server-anda:7880/health` untuk status kesehatan, atau gunakan dashboard Grafana kami di `http://alamat-server-anda:3001` untuk melihat grafik jumlah room, peer, dan bandwidth secara real-time.

---

## 📚 Langkah Selanjutnya
- Untuk memasang AlvioRelay di server Ubuntu/Debian produksi: baca **[Panduan Deployment Produksi](deployment.md)**.
- Untuk memantau grafik bandwidth & CPU server: baca **[Panduan Observability](observability.md)**.
- Untuk pengembang yang ingin membuat aplikasi Android/iOS/Web kustom: buka **[Jalur Developer](README.md#👨💻-2-jalur-developer-software-engineer--integrator)**.
