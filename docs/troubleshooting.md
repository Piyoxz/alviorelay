# Troubleshooting & Performance Tuning Guide

This document provides practical solutions for common WebRTC issues, operating system tuning for high-throughput media servers, and diagnostic workflows.

---

## 1. Linux Kernel Tuning (sysctl)

High-density WebRTC servers handle thousands of UDP packets per second. Default Linux socket buffer sizes can cause kernel-level packet drops before the application even reads them.

Add the following values to `/etc/sysctl.d/99-alvio-sfu.conf`:

```ini
# Maximum receive socket buffer size (32MB)
net.core.rmem_max = 33554432
net.core.rmem_default = 1048576

# Maximum send socket buffer size (32MB)
net.core.wmem_max = 33554432
net.core.wmem_default = 1048576

# Maximum network device backlog queue
net.core.netdev_max_backlog = 10000

# UDP memory limits (min, pressure, max in 4KB pages)
net.ipv4.udp_mem = 262144 524288 1048576

# File descriptor ceiling for high concurrency
fs.file-max = 2097152
```

Apply immediately without rebooting:
```bash
sudo sysctl --system
```

Also increase user file limits in `/etc/security/limits.d/99-alvio.conf`:
```text
alvio soft nofile 1048576
alvio hard nofile 1048576
```

---

## 2. Common Issues & Solutions

### 2.1 Video Screen is Black / Audio Silent (ICE Connection Failed)
- **Symptom**: The client connects to WebSocket signaling and says "Connected", but remote video never renders and audio is silent.
- **Root Cause**: The media plane UDP port is blocked by a firewall or router NAT, preventing WebRTC DTLS/SRTP handshake.
- **Diagnosis**:
  1. Open Chrome and navigate to `chrome://webrtc-internals`.
  2. Inspect the active `RTCIceCandidatePair`. Look for `iceState: "failed"` or `iceState: "checking"`.
- **Solution**:
  - Open **UDP port 7882** on your host/VPS firewall:
    ```bash
    sudo ufw allow 7882/udp
    ```
  - If running in Docker, verify `docker-compose.yml` has `"7882:7882/udp"`.
  - If running on AWS EC2 or Google Cloud behind a 1:1 NAT, set your public IP in `alvio-relay.toml`:
    ```toml
    [rtc]
    use_external_ip = true
    external_ip = "203.0.113.45"
    ```

---

### 2.2 Browser Refuses Camera / Microphone Permission
- **Symptom**: Browser shows *"NotAllowedError: Permission denied"* or never shows the permission popup.
- **Root Cause**: Modern browsers (Chrome, Safari, Firefox) restrict `navigator.mediaDevices.getUserMedia` to **Secure Contexts** only (`https://` or `localhost`).
- **Solution**:
  - For local development, access via `http://localhost:3000` (not via raw local IP like `http://192.168.1.50:3000`).
  - For production, place AlvioRelay behind a TLS reverse proxy (Caddy or Nginx) with a valid SSL certificate.

---

### 2.3 WebSocket Connection Drops or Fails with 400/502
- **Symptom**: Client fails to connect to `/ws`, receiving HTTP 400 Bad Request or 502 Bad Gateway.
- **Root Cause**: The reverse proxy is not forwarding the HTTP `Upgrade: websocket` and `Connection: Upgrade` headers.
- **Solution**:
  - In Nginx, ensure the following proxy directives are present:
    ```nginx
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    ```
  - In Caddy, `reverse_proxy` handles WebSockets automatically without extra configuration.

---

### 2.4 Audio Crackling / High Packet Loss
- **Symptom**: Audio breaks up, robotic speech sounds, or packet loss ratio exceeds 5%.
- **Root Cause**:
  - Network congestion on publisher or subscriber uplink.
  - Insufficient kernel UDP receive buffer (kernel dropping packets).
- **Solution**:
  - Check the Grafana dashboard for `alvio_packet_loss_ratio`.
  - Apply the `sysctl` kernel socket buffer tuning outlined in Section 1 above.
  - Enable Simulcast on publishers and verify that bandwidth estimators trigger layer down-switching (`"q"` or `"h"`).

---

## 3. Diagnostic Commands

### Check if AlvioRelay is Listening on UDP & TCP
```bash
sudo ss -tulpn | grep alvio
# Output should show:
# tcp LISTEN 0 511 0.0.0.0:7880
# udp UNCONN 0   0 0.0.0.0:7882
```

### Inspect Live Media Packet Counters
```bash
curl -s http://localhost:7880/metrics | grep -E "alvio_rtp|alvio_active"
```
