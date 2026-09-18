# AlvioRelay Flutter Plugin (`alvio_flutter`)

Cross-platform WebRTC audio/video calling and screen sharing for Flutter apps on iOS, Android, Windows, macOS, Linux, and Web.

## Features

- Single unified Dart API across all 6 Flutter platforms.
- Low-latency media routing with adaptive simulcast layer selection.
- Out-of-the-box support for Flutter WebRTC video renderers (`RTCVideoRenderer`).
- Real-time data channels for chat, whiteboard, and reactions.

## Quickstart

```dart
import 'package:flutter/material.dart';
import 'package:alvio_flutter/alvio_flutter.dart';

void main() => runApp(const AlvioApp());

class AlvioApp extends StatefulWidget {
  const AlvioApp({super.key});

  @override
  State<AlvioApp> createState() => _AlvioAppState();
}

class _AlvioAppState extends State<AlvioApp> {
  final _room = AlvioRoom();

  @override
  void initState() {
    super.initState();
    _initCall();
  }

  Future<void> _initCall() async {
    await _room.connect('wss://relay.alvio.io/ws');
    await _room.join(roomId: 'flutter-room', peerName: 'Flutter User');
  }

  @override
  void dispose() {
    _room.disconnect();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('AlvioRelay Flutter Call')),
        body: const Center(child: Text('Connected to Alvio SFU')),
      ),
    );
  }
}
```
