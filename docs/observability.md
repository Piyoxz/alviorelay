# Observability & Monitoring Guide

AlvioRelay exposes enterprise-grade metrics and health endpoints out of the box, ensuring seamless integration with Prometheus, Grafana, Datadog, and Kubernetes.

---

## 1. Endpoints Overview

| Endpoint | Method | Purpose | Response Format |
| :--- | :--- | :--- | :--- |
| `/metrics` | `GET` | Prometheus scraping target | Prometheus text format (`text/plain; version=0.0.4`) |
| `/health` | `GET` | Liveness probe | `application/json` (`{"status":"healthy"}`) |
| `/ready` | `GET` | Readiness probe | `application/json` (`{"status":"ready"}`) |

---

## 2. Prometheus Metric Catalog

### 2.1 Room & Peer Metrics
| Metric | Type | Description |
| :--- | :--- | :--- |
| `alvio_active_rooms` | Gauge | Total number of currently active rooms. |
| `alvio_active_peers` | Gauge | Total number of connected WebRTC participants across all rooms. |
| `alvio_peers_joined_total` | Counter | Cumulative count of peer join events. |
| `alvio_peers_left_total` | Counter | Cumulative count of peer disconnects or leaves. |

### 2.2 Media & SFU Routing Metrics
| Metric | Type | Description |
| :--- | :--- | :--- |
| `alvio_rtp_packets_in_total` | Counter | Total RTP media packets received from publishers. |
| `alvio_rtp_packets_out_total` | Counter | Total RTP media packets forwarded to subscribers. |
| `alvio_rtp_bytes_in_total` | Counter | Cumulative inbound media bytes. |
| `alvio_rtp_bytes_out_total` | Counter | Cumulative outbound forwarded media bytes. |
| `alvio_packet_loss_ratio` | Gauge | Instantaneous packet loss percentage across active connections. |

### 2.3 Resiliency & Quality Control Metrics
| Metric | Type | Description |
| :--- | :--- | :--- |
| `alvio_nack_requests_total` | Counter | Total NACK retransmission requests sent to publishers. |
| `alvio_pli_requests_total` | Counter | Total PLI (Picture Loss Indication) keyframe requests issued. |
| `alvio_simulcast_layer_switches_total` | Counter | Cumulative simulcast layer adaptations triggered by bandwidth estimators. |
| `alvio_data_channel_messages_total` | Counter | Total messages processed through SCTP Data Channels. |

---

## 3. Grafana Dashboard

AlvioRelay ships with a production-ready, dark-mode dashboard template located at:
```text
deploy/grafana/dashboards/alvio-overview.json
```

### Key Visualizations:
1. **Executive KPI Cards**: Active Rooms, Active Peers, Aggregate Inbound Bitrate (Mbps), Aggregate Outbound Bitrate (Mbps).
2. **Network Traffic Throughput**: Inbound vs Outbound packet rate (packets/sec) and bitrate graphs.
3. **Stream Quality & Health**: Packet loss trendline, NACK recovery frequency, and keyframe request rate.
4. **Data Channels Activity**: Message count per second and payload volume.

When running with `docker compose up -d`, Grafana is automatically available at `http://localhost:3001` with this dashboard pre-loaded.

---

## 4. Recommended Alerting Rules

Add these rules to your Prometheus alertmanager setup:

```yaml
groups:
  - name: alvio_relay_alerts
    rules:
      - alert: AlvioHighPacketLoss
        expr: alvio_packet_loss_ratio > 0.08
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Elevated packet loss on AlvioRelay instance {{ $labels.instance }}"
          description: "Packet loss has exceeded 8% for over 2 minutes."

      - alert: AlvioNodeDown
        expr: up{job="alvio-relay"} == 0
        for: 30s
        labels:
          severity: critical
        annotations:
          summary: "AlvioRelay server is down!"
          description: "Prometheus failed to scrape {{ $labels.instance }}."
```
