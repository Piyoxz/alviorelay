# AlvioRelay Windows Native Client SDK (`alvio-desktop-windows`)

High-performance native desktop client integration for Windows 10/11 (x64 and ARM64).

## Architecture & Features

The Windows SDK directly links to `alvio-client-core` via C FFI / ABI:
- **Low-Latency Audio**: Integration with Windows Audio Session API (WASAPI) Exclusive Mode for sub-10ms audio capture and playback with built-in acoustic echo cancellation (AEC).
- **GPU Desktop & Window Capture**: Direct3D 11 / DXGI Desktop Duplication API for hardware-accelerated 60fps 4K screen sharing with zero CPU copy overhead.
- **Hardware Video Acceleration**: DirectX Video Acceleration (DXVA2 / D3D11VA) hardware H.264 / AV1 / VP9 decoding and Intel QuickSync / NVIDIA NVENC encoding.
- **UI Frameworks Supported**: WinUI 3, Windows App SDK, WPF, and native Win32/C++.

## C / C++ FFI Header (`include/alvio_client.h`)

```c
#ifndef ALVIO_CLIENT_H
#define ALVIO_CLIENT_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct AlvioClientHandle AlvioClientHandle;

AlvioClientHandle* alvio_client_create(const char* version);
void alvio_client_destroy(AlvioClientHandle* handle);
int32_t alvio_client_state(AlvioClientHandle* handle);
int32_t alvio_client_join(AlvioClientHandle* handle, const char* room_id, const char* peer_name);
int32_t alvio_client_publish_track(AlvioClientHandle* handle, int32_t kind, const char* source);
int32_t alvio_client_send_data(AlvioClientHandle* handle, const char* payload, bool reliable);
int32_t alvio_client_leave(AlvioClientHandle* handle);
char* alvio_client_poll_event(AlvioClientHandle* handle);
void alvio_client_free_string(char* ptr);

#ifdef __cplusplus
}
#endif

#endif // ALVIO_CLIENT_H
```

## Quick Start (C++ Example)

```cpp
#include "alvio_client.h"
#include <iostream>

int main() {
    AlvioClientHandle* client = alvio_client_create("windows-desktop-1.0.0");
    if (!client) {
        std::cerr << "Failed to initialize Alvio client\n";
        return 1;
    }

    std::cout << "Client initialized. State: " << alvio_client_state(client) << "\n";

    // Clean up
    alvio_client_destroy(client);
    return 0;
}
```
