# Webhook & Security Guide

AlvioRelay features an asynchronous webhook engine (`alvio-hooks`) designed to notify your backend applications when key room and participant lifecycle events occur.

---

## 1. Webhook Configuration

Add the `[webhooks]` section to your `alvio-relay.toml`:

```toml
[webhooks]
url = "https://api.yourcompany.com/webhooks/alvio"
secret = "super-secret-hmac-key-change-in-production"
timeout_ms = 3000
max_retries = 3
```

---

## 2. Security Headers & Signature Verification

Every HTTP POST request sent by AlvioRelay contains cryptographic verification headers:

| Header Name | Description |
| :--- | :--- |
| `Alvio-Signature` | Hex-encoded HMAC-SHA256 signature calculated across the raw UTF-8 request body using your shared secret. |
| `Alvio-Timestamp` | Unix epoch timestamp (seconds) when the event was generated. |
| `Content-Type` | Always `application/json`. |

### Replay Attack Prevention
Your backend MUST check that `Math.abs(currentTime - Number(timestamp)) < 300` (within a 5-minute tolerance window) before verifying the signature.

---

## 3. Verification Code Examples

### 3.1 Node.js / TypeScript (Express)
```typescript
import crypto from "crypto";
import express from "express";

const app = express();
// IMPORTANT: Use express.raw() to preserve raw byte buffer for HMAC calculation
app.use(express.raw({ type: "application/json" }));

const WEBHOOK_SECRET = process.env.ALVIO_WEBHOOK_SECRET || "super-secret-hmac-key-change-in-production";

app.post("/webhooks/alvio", (req, res) => {
  const signature = req.headers["alvio-signature"] as string;
  const timestamp = req.headers["alvio-timestamp"] as string;

  if (!signature || !timestamp) {
    return res.status(401).send("Missing security headers");
  }

  // 1. Replay attack check (5-minute drift threshold)
  const now = Math.floor(Date.now() / 1000);
  if (Math.abs(now - parseInt(timestamp, 10)) > 300) {
    return res.status(401).send("Timestamp expired");
  }

  // 2. Compute HMAC-SHA256
  const hmac = crypto.createHmac("sha256", WEBHOOK_SECRET);
  hmac.update(req.body);
  const expectedSignature = hmac.digest("hex");

  // 3. Timing-safe comparison
  const isValid = crypto.timingSafeEqual(
    Buffer.from(signature, "hex"),
    Buffer.from(expectedSignature, "hex")
  );

  if (!isValid) {
    return res.status(403).send("Invalid HMAC signature");
  }

  const event = JSON.parse(req.body.toString("utf8"));
  console.log(`[Webhook Verified] Event: ${event.event_type} in room ${event.room_id}`);

  // Process event in your database or business logic
  res.status(200).json({ received: true });
});
```

### 3.2 Python (FastAPI)
```python
import hmac
import hashlib
import time
from fastapi import FastAPI, Request, HTTPException

app = FastAPI()
WEBHOOK_SECRET = b"super-secret-hmac-key-change-in-production"

@app.post("/webhooks/alvio")
async def handle_alvio_webhook(request: Request):
    signature = request.headers.get("alvio-signature")
    timestamp = request.headers.get("alvio-timestamp")
    
    if not signature or not timestamp:
        raise HTTPException(status_code=401, detail="Missing security headers")
        
    if abs(time.time() - int(timestamp)) > 300:
        raise HTTPException(status_code=401, detail="Timestamp expired")
        
    raw_body = await request.body()
    computed_hmac = hmac.new(WEBHOOK_SECRET, raw_body, hashlib.sha256).hexdigest()
    
    if not hmac.compare_digest(signature, computed_hmac):
        raise HTTPException(status_code=403, detail="Invalid HMAC signature")
        
    event = await request.json()
    return {"status": "ok", "event_id": event.get("event_id")}
```

---

## 4. Event Catalog

### `peer_joined`
```json
{
  "event_id": "evt-71932-a1",
  "event_type": "peer_joined",
  "room_id": "classroom-101",
  "timestamp": 1774003200,
  "payload": {
    "peer_id": "student-42",
    "display_name": "Sarah Connor",
    "role": "publisher"
  }
}
```

### `recording_completed`
```json
{
  "event_id": "evt-71933-b2",
  "event_type": "recording_completed",
  "room_id": "classroom-101",
  "timestamp": 1774006800,
  "payload": {
    "recording_id": "rec-2026-09-18-classroom-101",
    "duration_seconds": 3600,
    "file_path": "/app/recordings/rec-2026-09-18-classroom-101.mp4",
    "size_bytes": 451920384
  }
}
```
