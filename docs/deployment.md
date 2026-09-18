# Production Deployment Guide

This guide covers deploying AlvioRelay in mission-critical production environments, ranging from single-node VPS setups to scalable cloud deployments.

---

## 1. Hardware Sizing & Capacity Planning

Because AlvioRelay is written in 100% pure Rust using a zero-allocation, sans-I/O architecture, it requires significantly fewer resources than Java, Node.js, or Go-based alternatives:

| Concurrent Video Participants | Recommended vCPU | Recommended RAM | Network Uplink | Typical Memory Usage |
| :--- | :--- | :--- | :--- | :--- |
| **Up to 50** | 1 vCPU | 1 GB | 100 Mbps | ~45 MB |
| **50 to 250** | 2 vCPU | 2 GB | 500 Mbps | ~80 MB |
| **250 to 1,000** | 4 - 8 vCPU | 4 - 8 GB | 1 Gbps - 10 Gbps | ~180 MB |
| **1,000+** | Multi-node Cluster | 8+ GB / node | 10 Gbps | Distributed |

---

## 2. Option A: Docker Compose Deployment (Recommended)

Docker Compose provides an isolated, reproducible stack containing AlvioRelay, Prometheus metrics collector, and Grafana visualization.

### 1. Clone & Configure
```bash
git clone https://github.com/Piyoxz/alviorelay.git
cd alviorelay

# Edit configuration with your public IP if on cloud VPS
nano alvio-relay.toml
```

### 2. Launch Services
```bash
docker compose up -d
```

### 3. Verify Health
```bash
curl http://localhost:7880/health
# Response: {"status":"healthy"}
```

Services exposed:
- **AlvioRelay Control**: `http://YOUR_SERVER_IP:7880`
- **WebRTC UDP Media**: `udp://YOUR_SERVER_IP:7882`
- **Prometheus Metrics**: `http://YOUR_SERVER_IP:9090`
- **Grafana Dashboards**: `http://YOUR_SERVER_IP:3001` (login: `admin` / `alviosecure`)

---

## 3. Option B: Bare-Metal Linux Deployment (Systemd)

For maximum performance with zero containerization overhead.

### 1. Create Dedicated Service User
```bash
sudo useradd -r -s /bin/false alvio
```

### 2. Install Binary & Configuration
```bash
# Copy compiled release binary
sudo cp target/release/alvio-relay /usr/local/bin/
sudo chmod +x /usr/local/bin/alvio-relay

# Setup configuration and recording directories
sudo mkdir -p /etc/alvio /var/log/alvio /var/recordings/alvio
sudo cp alvio-relay.toml /etc/alvio/
sudo chown -R alvio:alvio /etc/alvio /var/log/alvio /var/recordings/alvio
```

### 3. Install Systemd Service Unit
Copy our production service file from `deploy/systemd/alvio-relay.service`:
```bash
sudo cp deploy/systemd/alvio-relay.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now alvio-relay
```

Check service status:
```bash
sudo systemctl status alvio-relay
```

---

## 4. Reverse Proxy & SSL/TLS Termination

WebRTC client browsers mandate HTTPS (`wss://`) for microphone and webcam access. Use Caddy or Nginx to terminate SSL.

### 4.1 Caddy (Automatic HTTPS in 3 Lines)
Install Caddy and place this into `/etc/caddy/Caddyfile`:

```caddyfile
relay.yourdomain.com {
    reverse_proxy 127.0.0.1:7880
}
```
Reload Caddy: `sudo systemctl reload caddy`. Caddy will automatically issue and renew a free Let's Encrypt certificate.

### 4.2 Nginx
Use our pre-configured template from `deploy/nginx/alvio.conf`:
```bash
sudo cp deploy/nginx/alvio.conf /etc/nginx/sites-available/relay.yourdomain.com
sudo ln -s /etc/nginx/sites-available/relay.yourdomain.com /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

---

## 5. Firewall & Network Configuration

Open the required incoming ports in your firewall:

### UFW (Ubuntu / Debian)
```bash
# Web, WebSocket Signaling, and HTTP API
sudo ufw allow 7880/tcp

# WebRTC UDP Media Traffic
sudo ufw allow 7882/udp

# Standard HTTPS / HTTP
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

sudo ufw reload
```

### AWS / DigitalOcean / Cloud Security Groups
| Type | Protocol | Port Range | Source | Description |
| :--- | :--- | :--- | :--- | :--- |
| Inbound | TCP | `80`, `443` | `0.0.0.0/0` | HTTP/HTTPS for Web Clients |
| Inbound | TCP | `7880` | `0.0.0.0/0` | Control Plane & Signaling |
| Inbound | UDP | `7882` | `0.0.0.0/0` | WebRTC Media Plane |
| Inbound | UDP | `50000-50200`| `0.0.0.0/0` | Multi-port dynamic allocation |
