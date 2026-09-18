# AlvioRelay iOS & macOS Client SDK (`alvio-ios`)

Official Swift SDK for iPhone, iPad, Apple Silicon Mac, and visionOS.

## Architecture

- **Core Engine**: Pre-built `AlvioClientCore.xcframework` packaged via Swift Package Manager (SPM).
- **AVFoundation Pipeline**: Native `AVCaptureSession` for 4K60 HDR video capture and zero-latency audio routing.
- **Metal Acceleration**: Hardware rendering of incoming remote streams using Metal shaders and `MTKView`.
- **System Integrations**:
  - **CallKit**: Native iOS lock screen incoming call UI and system Recents integration.
  - **AVAudioSession**: Seamless Bluetooth hands-free (HFP), AirPods spatial audio, and Carplay audio switching.
  - **Picture-in-Picture (PiP)**: `AVPictureInPictureVideoCallViewController` support for continuous video during multitasking.

## Swift Quickstart Example

```swift
import SwiftUI
import AlvioClient

class CallViewModel: ObservableObject {
    private let room = AlvioRoom()

    @Published var peers: [PeerInfo] = []
    @Published var isConnected = false

    func startCall() {
        room.onEvent = { [weak self] event in
            DispatchQueue.main.async {
                switch event {
                case .connected(let peerId, _):
                    self?.isConnected = true
                    self?.room.join(roomId: "room-ios", peerName: "iPhone User")
                case .peerJoined(let peer):
                    self?.peers.append(peer)
                case .peerLeft(let peerId, _):
                    self?.peers.removeAll { $0.id == peerId }
                default:
                    break
                }
            }
        }

        Task {
            try await room.connect(url: "wss://relay.alvio.io/ws", token: "jwt_token")
        }
    }
}
```
