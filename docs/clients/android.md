# Panduan Integrasi Client SDK (Mobile Android) — `alvio-android`

Panduan integrasi resmi untuk smartphone, tablet, foldable, dan Android TV menggunakan Kotlin / Java.

---

## 1. Arsitektur & Izin Aplikasi (Permissions)

Tambahkan izin berikut pada `AndroidManifest.xml`:

```xml
<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.RECORD_AUDIO" />
    <uses-permission android:name="android.permission.CAMERA" />
    <uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_CAMERA" />
    <uses-permission android:name="android.permission.FOREGROUND_SERVICE_MICROPHONE" />

    <application ...>
        <activity
            android:name=".CallActivity"
            android:supportsPictureInPicture="true"
            android:configChanges="screenSize|smallestScreenSize|screenLayout|orientation" />
    </application>
</manifest>
```

---

## 2. Inisialisasi AlvioRoom di Android

```kotlin
package com.example.alviocall

import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import io.alvio.client.AlvioRoom
import io.alvio.client.RoomEvent

class CallActivity : AppCompatActivity() {
    private lateinit var room: AlvioRoom

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_call)

        // Inisialisasi AlvioRoom dengan context Android
        room = AlvioRoom(applicationContext)

        // Observasi event ruangan
        room.onEvent { event ->
            runOnUiThread {
                when (event) {
                    is RoomEvent.Connected -> {
                        room.join(roomId = "ruang-medis-01", peerName = "Dr. Andi (Android)")
                    }
                    is RoomEvent.RoomJoined -> {
                        println("Berhasil bergabung ke ruangan: ${event.roomId}")
                    }
                    is RoomEvent.TrackPublished -> {
                        // Pasang track video remote ke SurfaceViewRenderer
                    }
                    is RoomEvent.PeerLeft -> {
                        println("Peer keluar: ${event.peerId}")
                    }
                    else -> {}
                }
            }
        }

        // Mulai koneksi WebSocket ke Signaling Gateway
        room.connect("wss://relay.alvio.io/ws", authToken = null)
    }

    override fun onUserLeaveHint() {
        super.onUserLeaveHint()
        // Masuk ke mode Picture-in-Picture secara otomatis saat pengguna menekan tombol Home
        enterPictureInPictureMode()
    }

    override fun onDestroy() {
        super.onDestroy()
        room.disconnect()
    }
}
```

---

## 3. Audio & Manajemen Rute Perangkat Keras

Alvio Android SDK mengelola rute audio secara otomatis melalui `AudioManager`:
- Beralih otomatis antara Speakerphone, Earpiece, Headphone Kabel, dan Bluetooth SCO / LE Audio.
- Mengaktifkan Acoustic Echo Canceler (AEC) bawaan hardware melalui `android.media.audiofx.AcousticEchoCanceler`.
