# ADR-0002: WebRTC Engine Selection & Sans-I/O Architecture

## Status
Accepted

## Context
High-performance SFU servers live and die by their media hot path latency and predictability. Standard WebRTC libraries (such as `webrtc-rs`, which ports Go Pion) embed internal async tasks, mutexes, and socket loops inside the library. This creates lock contention under high subscriber counts and prevents deterministic packet scheduling.

## Decision
We select **`str0m`** as our primary WebRTC engine, wrapped strictly behind AlvioRelay's internal `AlvioTransport` abstraction:
1. **Sans-I/O Pattern**: `str0m` is a pure state machine with zero network sockets and zero internal threads.
2. **Deterministic & Testable**: WebRTC ICE, DTLS, and SRTP handshakes can be driven in unit tests deterministically by passing byte slices between instances without any network hardware.
3. **Lockless UDP Loop**: The packet receive loop (`recvmmsg`) in `alvio-sfu` controls I/O directly without library-level locks.
4. **Isolating Abstraction**: Crate `alvio-webrtc` exposes `AlvioTransport`, ensuring that future upgrades or custom protocol components do not leak third-party types into the SFU router.

## Consequences
- **Positive**: Exceptional CPU cache efficiency, zero thread contention in media pipeline, ultra-low latency.
- **Negative**: Requires managing I/O timers and socket loops explicitly at the application layer.
