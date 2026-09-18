# alvio-client-core

> **Cross-platform core client engine and C-ABI FFI bindings for AlvioRelay (Desktop, Mobile, Embedded).**

[![Documentation](https://docs.rs/alvio-client-core/badge.svg)](https://docs.rs/alvio-client-core)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

`alvio-client-core` is the native Rust engine powering AlvioRelay native client SDKs on **Windows (WinUI/WASAPI)**, **macOS (Swift/Metal)**, **Linux (GTK/PipeWire)**, **Android (Kotlin JNI)**, **iOS (Swift C-ABI)**, **Flutter (Dart FFI)**, and **React Native (TurboModules)**.

---

## Features

- **Async Client Lifecycle**: Complete state machine (`Disconnected`, `Connecting`, `Connected`, `InCall`, `Reconnecting`).
- **C-ABI FFI Header**: `include/alvio_client.h` exports stable C functions for easy foreign-function integration in any language.
- **Background Event Pump**: Spawns a Tokio background worker, notifying host platforms via thread-safe callbacks.
- **SCTP Data Channels API**: Send and receive reliable or lossy data messages natively.

---

## C-ABI Header (`include/alvio_client.h`)

```c
#include <stdint.h>
#include <stdbool.h>

typedef void* AlvioClientHandle;

AlvioClientHandle alvio_client_create(
    const char* server_url,
    const char* room_id,
    const char* peer_id,
    const char* display_name
);

int32_t alvio_client_connect(AlvioClientHandle handle);
int32_t alvio_client_publish_track(AlvioClientHandle handle, const char* track_id, const char* kind);
int32_t alvio_client_send_data(AlvioClientHandle handle, const uint8_t* data, size_t len, bool reliable);
int32_t alvio_client_disconnect(AlvioClientHandle handle);
void alvio_client_destroy(AlvioClientHandle handle);
```

---

## License

Dual-licensed under either Apache-2.0 or MIT.
