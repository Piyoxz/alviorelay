# ADR-0001: Modular Rust Workspace Architecture

## Status
Accepted

## Context
AlvioRelay is designed as an independent, self-hostable, production-grade realtime media infrastructure. Many existing media servers either bundle everything into a monolithic codebase (making embedded or custom use difficult) or over-abstract with microservices that require complex orchestration just to run locally.

## Decision
We adopt a **Rust Virtual Workspace** structure composed of decoupled crates with single responsibilities:
- `alvio-core`: Shared core domain models and error types.
- `alvio-protocol`: Signaling schemas and serialization format.
- `alvio-webrtc`: WebRTC low-level primitives wrapped behind `AlvioTransport`.
- `alvio-sfu`: Selective Forwarding Unit media routing and feedback engine.
- `alvio-signal`: WebSocket session and room coordination state machine.
- `alvio-auth`: Pluggable authentication traits.
- `alvio-storage`: Pluggable object/local storage backend traits.
- `alvio-hooks`: Webhook dispatch engine with HMAC signing.
- `alvio-observe`: Prometheus metrics, tracing, and health telemetry.
- `alvio-egress`: Out-of-process media recording supervisor.
- `alvio-ingress`: WHIP and external ingest bridges.
- `services/relay`: The single deployable server binary (`alvio-relay`).

## Consequences
- **Positive**: Clean boundaries, easy unit testing of isolated components, ability to reuse crates in external tools/SDKs, fast incremental builds.
- **Negative**: Requires maintaining workspace dependency versions and clean boundary APIs across crates.
