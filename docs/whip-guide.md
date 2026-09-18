# WHIP Ingestion Guide (OBS Studio & FFmpeg)

AlvioRelay provides first-class support for **WHIP (WebRTC-HTTP Ingestion Protocol)**, allowing professional broadcasting tools like **OBS Studio**, **vMix**, and **FFmpeg** to stream high-definition video directly into an AlvioRelay room with sub-second latency.

---

## 1. WHIP Endpoint Overview

- **Protocol**: HTTP/1.1 or HTTP/2 over TLS (HTTPS).
- **HTTP Method**: `POST`
- **URL Path**: `/whip/{room_id}`
- **Request Headers**:
  - `Content-Type: application/sdp`
  - `Authorization: Bearer <JWT_TOKEN>` *(optional, if authentication is configured)*
- **Response**:
  - `201 Created` with `Content-Type: application/sdp` containing the SFU's SDP Answer.
  - `Location: /whip/{room_id}/{session_id}` header for stream termination via `DELETE`.

---

## 2. OBS Studio Setup (Step-by-Step)

OBS Studio version 30.0 and newer includes native WHIP support out of the box.

1. Open **OBS Studio** and navigate to **Settings** (`Ctrl+,` or `Cmd+,`).
2. Select the **Stream** tab on the left.
3. Configure the stream options:
   - **Service**: Select **WHIP**.
   - **Server**: Enter your AlvioRelay WHIP URL:
     ```text
     https://relay.yourdomain.com/whip/live-stage-01
     ```
     *(For local testing on your machine: `http://localhost:7880/whip/test-room`)*.
   - **Bearer Token**: Enter your JWT authentication token (leave blank if running with `provider = "no_auth"`).
4. Select the **Output** tab to optimize for real-time WebRTC:
   - **Output Mode**: Advanced
   - **Encoder**: NVIDIA NVENC H.264, Apple VideoToolbox, or x264
   - **Rate Control**: CBR (Constant Bitrate)
   - **Bitrate**: `2500 Kbps` to `6000 Kbps`
   - **Keyframe Interval**: `1s` or `2s` *(Essential for WebRTC keyframe sync)*
   - **Tune**: Low Latency / Ultra Low Latency
5. Click **Apply** and **OK**.
6. Click **Start Streaming**. The video will immediately appear in the room `live-stage-01` and be distributed to all subscribers in real-time.

---

## 3. Streaming with FFmpeg

You can publish test patterns, pre-recorded video files, or live camera inputs using modern FFmpeg builds with WebRTC output:

```bash
# Stream an MP4 video in a continuous loop to AlvioRelay via WHIP
ffmpeg -re -stream_loop -1 -i test_video.mp4 \
  -c:v libx264 -preset ultrafast -tune zerolatency -b:v 2500k -g 30 \
  -c:a aac -b:a 128k \
  -f webrtc "http://localhost:7880/whip/stage-video"
```

---

## 4. Programmatic HTTP Request (Curl)

If you are developing a custom hardware encoder or native client:

```bash
curl -X POST "http://localhost:7880/whip/webinar-room" \
  -H "Content-Type: application/sdp" \
  --data-binary @offer.sdp
```

The server returns HTTP `201 Created` with the SDP Answer in the response body.
