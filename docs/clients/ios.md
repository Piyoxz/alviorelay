# Panduan Integrasi Client SDK (Mobile iOS) — `alvio-ios`

Panduan integrasi resmi untuk iPhone, iPad, dan Apple Vision Pro menggunakan Swift dan SwiftUI.

---

## 1. Setup Info.plist & Privacy Keys

Tambahkan kunci privasi kamera dan mikrofon pada `Info.plist`:

```xml
<key>NSCameraUsageDescription</key>
<string>Aplikasi memerlukan akses kamera untuk panggilan video interaktif.</string>
<key>NSMicrophoneUsageDescription</key>
<string>Aplikasi memerlukan akses mikrofon untuk komunikasi suara.</string>
<key>UIBackgroundModes</key>
<array>
    <string>audio</string>
    <string>voip</string>
</array>
```

---

## 2. Inisialisasi & Penggunaan di SwiftUI

```swift
import SwiftUI
import AlvioClient

struct VideoCallView: View {
    @StateObject private var callManager = CallManager()

    var body: some View {
        VStack {
            Text("Status: \(callManager.statusText)")
                .font(.headline)

            if callManager.isInRoom {
                Text("Terhubung ke ruangan: \(callManager.roomId)")
                Button("Keluar Ruangan") {
                    Task {
                        await callManager.leaveCall()
                    }
                }
                .buttonStyle(.borderedProminent)
                .tint(.red)
            } else {
                Button("Gabung Ruangan") {
                    Task {
                        await callManager.joinCall()
                    }
                }
                .buttonStyle(.borderedProminent)
            }
        }
        .padding()
        .task {
            await callManager.initialize()
        }
    }
}

@MainActor
class CallManager: ObservableObject {
    private let room = AlvioRoom()
    @Published var statusText = "Disconnected"
    @Published var isInRoom = false
    @Published var roomId = "room-ios-demo"

    func initialize() async {
        room.onEvent = { [weak self] event in
            guard let self = self else { return }
            switch event {
            case .connected(let peerId, _):
                self.statusText = "Connected (\(peerId))"
            case .roomJoined(let rId, _, _):
                self.isInRoom = true
                self.roomId = rId
                self.statusText = "In Room"
            case .disconnected:
                self.isInRoom = false
                self.statusText = "Disconnected"
            default:
                break
            }
        }

        try? await room.connect(url: "wss://relay.alvio.io/ws")
    }

    func joinCall() async {
        try? await room.join(roomId: roomId, peerName: "Pengguna iPhone")
    }

    func leaveCall() async {
        try? await room.leave()
    }
}
```

---

## 3. Integrasi CallKit & AVAudioSession

- Gunakan `CXProvider` untuk menampilkan antarmuka panggilan layar kunci iOS (*native system in-call UI*).
- Konfigurasi `AVAudioSession.sharedInstance().setCategory(.playAndRecord, mode: .videoChat, options: [.allowBluetooth, .allowBluetoothA2DP])`.
