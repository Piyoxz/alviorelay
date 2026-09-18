# Panduan Integrasi Client SDK (Cross-Platform Flutter) — `alvio_flutter`

Panduan integrasi multiplatform satu basis kode untuk iOS, Android, Windows, macOS, Linux, dan Web menggunakan Flutter.

---

## 1. Instalasi Dependensi

Tambahkan dependensi ke `pubspec.yaml`:

```yaml
dependencies:
  flutter:
    sdk: flutter
  alvio_flutter:
    path: clients/flutter
  flutter_webrtc: ^0.11.7
```

---

## 2. Inisialisasi Ruangan & Render Video

```dart
import 'package:flutter/material.dart';
import 'package:alvio_flutter/alvio_flutter.dart';
import 'package:flutter_webrtc/flutter_webrtc.dart';

class RoomScreen extends StatefulWidget {
  final String roomId;
  final String userName;

  const RoomScreen({super.key, required this.roomId, required this.userName});

  @override
  State<RoomScreen> createState() => _RoomScreenState();
}

class _RoomScreenState extends State<RoomScreen> {
  late final AlvioRoom _room;
  final RTCVideoRenderer _localRenderer = RTCVideoRenderer();

  @override
  void initState() {
    super.initState();
    _initRoom();
  }

  Future<void> _initRoom() async {
    await _localRenderer.initialize();
    _room = AlvioRoom();

    _room.on('connected', (data) {
      _room.join(roomId: widget.roomId, peerName: widget.userName);
    });

    _room.on('roomJoined', (data) async {
      // Dapatkan stream lokal dan pasang ke renderer
      final mediaStream = await navigator.mediaDevices.getUserMedia({
        'audio': true,
        'video': {'facingMode': 'user'},
      });
      _localRenderer.srcObject = mediaStream;
      setState(() {});
    });

    await _room.connect('wss://relay.alvio.io/ws');
  }

  @override
  void dispose() {
    _localRenderer.dispose();
    _room.disconnect();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text('Ruangan: ${widget.roomId}')),
      body: Center(
        child: RTCVideoView(_localRenderer, mirror: true),
      ),
    );
  }
}
```
