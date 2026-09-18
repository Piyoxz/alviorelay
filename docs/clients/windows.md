# Panduan Integrasi Client SDK (Desktop Windows) — `alvio-desktop-windows`

Panduan komprehensif bagi developer aplikasi desktop Windows native (.exe, WinUI 3, WPF, C++, Rust, C#).

---

## 1. Arsitektur Native Windows

SDK Desktop Windows AlvioRelay dirancang khusus untuk performa ekstrem dengan latensi mendekati perangkat keras (*bare-metal latency*):

1. **Audio Pipeline**: Menggunakan **Windows Audio Session API (WASAPI)** Exclusive Mode. Mendukung hardware endpoint capture dengan buffer serendah 3–10 ms, bypass software mixer Windows, dan isolasi thread audio bertaraf *Pro Audio MMCSS (Multimedia Class Scheduler Service)*.
2. **Screen & Window Capture**: Menggunakan **DirectX Graphics Infrastructure (DXGI) Desktop Duplication API** dan Windows Graphics Capture (WGC). Frame GPU langsung disalin ke tekstur Direct3D 11 tanpa transfer memori CPU, mampu menangani 4K 60fps dengan utilisasi CPU di bawah 2%.
3. **Hardware Video Codec**: Terhubung langsung ke **Intel Quick Sync Video (QSV)**, **NVIDIA NVENC**, atau **AMD AMF** melalui DirectX Video Acceleration (DXVA / D3D11VA) untuk encoding H.264 / AV1 hemat daya.

---

## 2. Pilihan Bahasa & Integrasi

### A. Integrasi Melalui Rust Native (Direkomendasikan)
Tambahkan dependensi langsung di `Cargo.toml`:
```toml
[dependencies]
alvio-client-core = { path = "../crates/alvio-client-core" }
tokio = { version = "1.40", features = ["full"] }
```

Contoh pemanggilan:
```rust
use alvio_client_core::{ClientRoom, ClientEvent, ConnectionState};
use alvio_core::{RoomId, StreamKind, StreamLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let room = ClientRoom::new("windows-client-1.0.0");
    let mut events = room.subscribe_events();

    // Loop penanganan event UI
    tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            match event {
                ClientEvent::Connected { peer_id, node_id } => {
                    println!("Terhubung ke node {} sebagai {}", node_id, peer_id);
                }
                ClientEvent::RoomJoined { room_id, peers, .. } => {
                    println!("Bergabung ke ruangan {room_id:?} dengan {} peers", peers.len());
                }
                ClientEvent::DataReceived { source_peer_id, payload } => {
                    println!("Data masuk dari {}: {}", source_peer_id, payload);
                }
                _ => {}
            }
        }
    });

    Ok(())
}
```

---

### B. Integrasi Melalui C / C++ / WinUI 3 via FFI

Gunakan header C `alvio_client.h` dan tautkan `alvio_client_core.lib` / `alvio_client_core.dll`:

```cpp
#include <windows.h>
#include <iostream>
#include "alvio_client.h"

int main() {
    // 1. Inisialisasi engine client
    AlvioClientHandle* client = alvio_client_create("winui-desktop-app-1.0");
    if (!client) {
        std::cerr << "Gagal membuat Alvio client handle\n";
        return -1;
    }

    // 2. Gabung ke ruangan
    int res = alvio_client_join(client, "ruang-rapat-dev", "Presenter Windows");
    if (res == 0) {
        std::cout << "Request bergabung ke ruangan terkirim!\n";
    }

    // 3. Publikasikan track mikrofon (0 = Audio)
    alvio_client_publish_track(client, 0, "wasapi_microphone");

    // 4. Loop polling event untuk pembaruan UI Win32
    while (true) {
        char* event_json = alvio_client_poll_event(client);
        if (event_json != nullptr) {
            std::cout << "Event UI: " << event_json << "\n";
            alvio_client_free_string(event_json);
        }
        Sleep(16); // ~60fps UI tick
    }

    // 5. Bersihkan saat keluar
    alvio_client_destroy(client);
    return 0;
}
```

---

## 3. Optimasi Latensi Windows

Untuk menjamin latency minimal saat streaming:
- **Aktifkan MMCSS**:
  ```cpp
  DWORD taskIndex = 0;
  HANDLE hTask = AvSetMmThreadCharacteristics(TEXT("Pro Audio"), &taskIndex);
  ```
- **Matikan Windows Nagle Algorithm** pada soket TCP signaling.
- **Konfigurasi DXGI Frame Latency**: Set `IDXGIDevice1::SetMaximumFrameLatency(1)` untuk mencegah antrian buffer frame video.
