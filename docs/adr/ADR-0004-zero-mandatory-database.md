# ADR-0004: Zero Mandatory Database & In-Memory State Model

## Status
Accepted

## Context
A major operational barrier for self-hosting realtime media infrastructure is the requirement to configure and maintain external databases (PostgreSQL, MySQL, Redis, MongoDB). For realtime media, room sessions and participant subscriptions are ephemeral by nature—persisting them to disk or an external store on every state transition introduces I/O latency and failure modes.

## Decision
AlvioRelay will **NOT** require any database to function:
1. Room and Peer states live 100% in-memory using concurrent, lock-free or read-mostly data structures (`DashMap` / `ArcSwap`).
2. When a room ends and its empty timeout expires, its in-memory resources are cleanly reclaimed.
3. Media hot path (packet routing) NEVER touches any database or blocking I/O.
4. Database integrations (e.g. archiving room session metrics, audit logs) are purely optional plugins on the control plane via hooks or external adapters, never on the media hot path.

## Consequences
- **Positive**: Instant self-hosting via `docker run` or single binary without dependencies; maximum media throughput and zero DB connection pool bottlenecks.
- **Negative**: Room state does not survive sudden process crashes; clients rely on standard WebRTC reconnection and session recovery.
