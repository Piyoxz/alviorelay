# ADR-0003: Versioned WebSocket Signaling Protocol

## Status
Accepted

## Context
WebRTC requires an out-of-band signaling mechanism to exchange SDP offers, answers, ICE candidates, and room state transitions. Proprietary or ad-hoc protocols often tightly couple internal server structs to the wire format, breaking backwards compatibility whenever internal fields change.

## Decision
We define an independent, versioned JSON WebSocket signaling protocol (`version: 1`) encapsulated in `alvio-protocol`:
1. Every message has an envelope: `version` (integer), `id` (unique tracking string), `type` (message discriminator), and `payload` (typed object).
2. Wire message types are explicitly defined and never expose internal Rust structs.
3. Message types include: `connect`, `ack`, `join`, `room_joined`, `offer`, `answer`, `candidate`, `publish`, `track_published`, `subscribe`, `layer_select`, `data`, `leave`.
4. The schema is documented formally so that SDKs in any language (TypeScript, Rust, Go, Python) can implement it without reverse-engineering server code.

## Consequences
- **Positive**: Clean protocol versioning, robust error handling, interoperable across multi-language clients.
- **Negative**: Slight serialization overhead compared to raw Protobuf/binary, but negligible for signaling control plane rates.
