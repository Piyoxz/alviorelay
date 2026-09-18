# AlvioRelay Signaling Protocol Specification (v1)

This specification defines the JSON-based signaling protocol (version 1) used by AlvioRelay for room lifecycle management, WebRTC peer negotiation, track announcements, simulcast switching, and data channel orchestration.

---

## 1. Transport & Connection Lifecycle

- **Transport**: WebSocket (`ws://` for local development, `wss://` for production).
- **Endpoint**:
  ```http
  GET /ws?room_id={room_id}&peer_id={peer_id}&token={auth_token}
  ```
- **Query Parameters**:
  | Parameter | Type | Required | Description |
  | :--- | :--- | :--- | :--- |
  | `room_id` | String | Yes | Target room identifier (alphanumeric, dashes, underscores). |
  | `peer_id` | String | Yes | Unique identifier for the connecting participant. |
  | `token` | String | Optional | Bearer JWT token if auth provider is enabled (`jwt`). |

---

## 2. Universal Message Envelope

All signaling messages sent over the WebSocket connection MUST conform to the standard JSON envelope:

```json
{
  "version": 1,
  "type": "string",
  "room_id": "string",
  "peer_id": "string",
  "payload": {}
}
```

- `version` *(integer, required)*: Must be `1`.
- `type` *(string, required)*: The specific message type identifier.
- `room_id` *(string, required)*: ID of the active room.
- `peer_id` *(string, required)*: ID of the sending peer (or server node).
- `payload` *(object, required)*: Type-specific JSON object.

---

## 3. Client-to-Server Messages (Upstream)

### 3.1 `join`
Sent immediately after WebSocket connection is established to declare participant identity and capabilities.

```json
{
  "version": 1,
  "type": "join",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "display_name": "Alice Developer",
    "role": "publisher",
    "metadata": {
      "avatar_url": "https://example.com/avatar.png",
      "platform": "web"
    }
  }
}
```

### 3.2 `offer`
Sent when the client initiates an SDP offer for publishing or subscribing.

```json
{
  "version": 1,
  "type": "offer",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "sdp": "v=0\r\no=- 42001 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0 1\r\n..."
  }
}
```

### 3.3 `answer`
Sent in response to a server-initiated renegotiation offer.

```json
{
  "version": 1,
  "type": "answer",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "sdp": "v=0\r\no=- 42002 2 IN IP4 127.0.0.1\r\ns=-\r\n..."
  }
}
```

### 3.4 `candidate`
Trickle ICE candidate exchange.

```json
{
  "version": 1,
  "type": "candidate",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "candidate": "candidate:842163049 1 udp 1677729535 192.168.1.100 54321 typ host",
    "sdp_mid": "0",
    "sdp_mline_index": 0
  }
}
```

### 3.5 `track_published`
Announces that the peer has started sending a new audio or video track.

```json
{
  "version": 1,
  "type": "track_published",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "track_id": "trk-video-camera-01",
    "kind": "video",
    "ssrc": 100101,
    "simulcast_layers": [
      { "rid": "q", "max_bitrate_bps": 150000, "scale_down_by": 4.0 },
      { "rid": "h", "max_bitrate_bps": 500000, "scale_down_by": 2.0 },
      { "rid": "f", "max_bitrate_bps": 1500000, "scale_down_by": 1.0 }
    ]
  }
}
```

### 3.6 `layer_switch`
Requests the SFU to dynamically switch the forwarded simulcast layer for a remote track.

```json
{
  "version": 1,
  "type": "layer_switch",
  "room_id": "conference-alpha",
  "peer_id": "user-bob",
  "payload": {
    "track_id": "trk-video-camera-01",
    "target_layer": "h"
  }
}
```
*Layers: `"q"` (Quarter/Low), `"h"` (Half/Medium), `"f"` (Full/High).*

### 3.7 `leave`
Gracefully signals departure from the room.

```json
{
  "version": 1,
  "type": "leave",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "reason": "user_hangup"
  }
}
```

---

## 4. Server-to-Client Messages (Downstream)

### 4.1 `joined`
Confirmation sent to the peer upon successfully joining a room.

```json
{
  "version": 1,
  "type": "joined",
  "room_id": "conference-alpha",
  "peer_id": "user-alice",
  "payload": {
    "room_id": "conference-alpha",
    "session_id": "sess-8f921d-99",
    "existing_peers": [
      {
        "peer_id": "user-bob",
        "display_name": "Bob Senior",
        "tracks": [
          { "track_id": "trk-audio-bob-01", "kind": "audio" },
          { "track_id": "trk-video-bob-01", "kind": "video" }
        ]
      }
    ],
    "ice_servers": [
      { "urls": ["stun:stun.l.google.com:19302"] }
    ]
  }
}
```

### 4.2 `peer_joined` & `peer_left`
Broadcast to all room members when a participant enters or leaves.

```json
{
  "version": 1,
  "type": "peer_joined",
  "room_id": "conference-alpha",
  "peer_id": "user-bob",
  "payload": {
    "peer_id": "user-bob",
    "display_name": "Bob Senior"
  }
}
```

```json
{
  "version": 1,
  "type": "peer_left",
  "room_id": "conference-alpha",
  "peer_id": "user-bob",
  "payload": {
    "peer_id": "user-bob",
    "reason": "timeout"
  }
}
```

### 4.3 `error`
Standardized error envelope returned when a signaling or state error occurs.

```json
{
  "version": 1,
  "type": "error",
  "room_id": "conference-alpha",
  "peer_id": "server",
  "payload": {
    "code": "ROOM_CAPACITY_EXCEEDED",
    "message": "Maximum room limit of 500 participants reached.",
    "retryable": false
  }
}
```

---

## 5. Heartbeat & Connection Health

- Clients SHOULD send a lightweight `{ "version": 1, "type": "ping", ... }` every **15 seconds**.
- The server responds with `{ "version": 1, "type": "pong", ... }`.
- Connections with no activity or pong response for **35 seconds** are terminated with WebRTC connection cleanup and webhook event dispatch.
