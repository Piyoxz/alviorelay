# Panduan Integrasi Client SDK (macOS & Linux) — `alvio-desktop-unix`

Panduan integrasi client desktop native untuk macOS (Swift, Metal, CoreMedia) dan Linux (GTK, Tauri, PipeWire, PulseAudio).

---

## 1. macOS Native Architecture

Pada macOS, SDK memanfaatkan kemampuan akselerasi Apple Silicon (M1/M2/M3/M4):
- **Video Capture & Processing**: `AVFoundation` dengan `AVCaptureScreenInput` / `ScreenCaptureKit` (macOS 12.3+) untuk rekaman layar resolusi retina dengan audio aplikasi terpisah.
- **Hardware Acceleration**: VideoToolbox hardware encoder H.264/HEVC/ProRes dan Metal Shading Language (`MTKView`) untuk render rendering zero-copy.
- **Audio Session**: `CoreAudio` AudioUnits dengan latensi buffer I/O hingga 1.5ms.

---

## 2. Linux Native Architecture (GTK / Tauri / PipeWire)

Pada distribusi Linux modern (Ubuntu, Fedora, Arch, Debian):
- **Audio Pipeline**: Terhubung dengan **PipeWire** (atau PulseAudio fallback) melalui SPA (Simple Plugin API) untuk latency audio profesional tanpa buffer xruns.
- **Screen Share**: Integrasi **XDG Desktop Portal** (`org.freedesktop.portal.ScreenCast`) melalui Wayland / PipeWire video stream.
- **GUI Framework**: Dapat dikompilasi ke aplikasi native menggunakan **Tauri v2**, **GTK 4 / Libadwaita**, atau **Qt 6**.

---

## 3. Contoh Integrasi macOS (Swift)

```swift
import Foundation
import ScreenCaptureKit
import AlvioClient

class MacPresenterViewModel {
    private let room = AlvioRoom()

    func connectAndShareScreen() async throws {
        try await room.connect(url: "wss://relay.alvio.io/ws")
        try await room.join(roomId: "mac-studio-room", peerName: "Mac Studio M3 Max")

        // Inisialisasi ScreenCaptureKit untuk menangkap jendela aktif
        let shareableContent = try await SCShareableContent.current
        if let display = shareableContent.displays.first {
            print("Berbagi layar tampilan utama: \(display.width)x\(display.height)")
            // Kirim stream ke engine Alvio
        }
    }
}
```

---

## 4. Contoh Integrasi Linux (Rust & PipeWire)

```rust
use alvio_client_core::ClientRoom;
use alvio_core::{RoomId, StreamKind, StreamLayer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let room = ClientRoom::new("linux-desktop-1.0.0");
    println!("Alvio Client Linux berhasil diinisialisasi.");
    Ok(())
}
```
