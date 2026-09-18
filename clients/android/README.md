# AlvioRelay Android Native Client SDK (`alvio-android`)

Official Android SDK for phone, tablet, and Android TV real-time video conferencing.

## Architecture

- **Core Engine**: Powered by `alvio-client-core` compiled via Android NDK for `arm64-v8a`, `armeabi-v7a`, `x86_64`.
- **Camera Pipeline**: Modern `Camera2` & `CameraX` API integration with auto-focus, exposure control, and multi-lens switching (Ultra-wide, Telephoto, Front).
- **Audio Subsystem**: High-performance low-latency audio via `AAudio` / `Oboe` with hardware Acoustic Echo Cancellation (AEC) and Noise Suppression (NS).
- **Background Calling & PiP**: Android `ForegroundService` with microphone/camera permissions, Picture-in-Picture (`android:supportsPictureInPicture="true"`), and `Telecom` framework incoming call notifications.

## Kotlin Quickstart Example

```kotlin
import io.alvio.client.AlvioRoom
import io.alvio.client.RoomEvent

class VideoCallActivity : AppCompatActivity() {
    private lateinit var room: AlvioRoom

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_video_call)

        room = AlvioRoom(context = this)

        room.onEvent { event ->
            when (event) {
                is RoomEvent.Connected -> {
                    room.join(roomId = "room-demo", peerName = "AndroidUser")
                }
                is RoomEvent.TrackPublished -> {
                    // Attach remote video track to SurfaceViewRenderer
                }
                is RoomEvent.DataReceived -> {
                    // Show incoming chat message
                }
            }
        }

        room.connect(serverUrl = "wss://relay.alvio.io/ws", token = "jwt_token")
    }

    override fun onDestroy() {
        super.onDestroy()
        room.disconnect()
    }
}
```
