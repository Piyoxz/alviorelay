# Panduan Integrasi Client SDK (React Native) — `@alviorelay/react-native`

Panduan integrasi framework React Native untuk iOS dan Android.

---

## 1. Instalasi

```bash
npm install @alviorelay/react-native @alviorelay/client react-native-webrtc
# atau
yarn add @alviorelay/react-native @alviorelay/client react-native-webrtc
```

Pastikan CocoaPods telah terpasang pada iOS:
```bash
cd ios && pod install && cd ..
```

---

## 2. Contoh Komponen Video Call React Native

```tsx
import React, { useEffect, useState } from 'react';
import { StyleSheet, View, Text, TouchableOpacity } from 'react-native';
import { AlvioRoom } from '@alviorelay/client';
import { RTCView, mediaDevices } from 'react-native-webrtc';

export function VideoMeetingRoom({ route }: { route: any }) {
  const [room] = useState(() => new AlvioRoom());
  const [localStreamUrl, setLocalStreamUrl] = useState<string | null>(null);
  const [peerCount, setPeerCount] = useState(0);

  useEffect(() => {
    room.on('connected', () => {
      room.join('meeting-rn', 'Pengguna React Native');
    });

    room.on('roomJoined', async (data) => {
      setPeerCount(data.peers.length);

      // Tangkap kamera depan ponsel
      const stream = await mediaDevices.getUserMedia({
        audio: true,
        video: { facingMode: 'user' },
      });
      setLocalStreamUrl(stream.toURL());
    });

    room.on('peerJoined', () => setPeerCount((prev) => prev + 1));
    room.on('peerLeft', () => setPeerCount((prev) => Math.max(0, prev - 1)));

    room.connect('wss://relay.alvio.io/ws');

    return () => {
      room.disconnect();
    };
  }, [room]);

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Peserta Aktif: {peerCount}</Text>
      {localStreamUrl && (
        <RTCView
          streamURL={localStreamUrl}
          style={styles.videoStream}
          mirror={true}
          objectFit="cover"
        />
      )}
      <TouchableOpacity
        style={styles.hangupButton}
        onPress={() => room.disconnect()}
      >
        <Text style={styles.buttonText}>Tutup Panggilan</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: { flex: 1, backgroundColor: '#121212', alignItems: 'center', justifyContent: 'center' },
  title: { color: '#ffffff', fontSize: 18, marginBottom: 16 },
  videoStream: { width: 320, height: 240, borderRadius: 12, backgroundColor: '#000' },
  hangupButton: { marginTop: 24, padding: 14, backgroundColor: '#e53935', borderRadius: 8 },
  buttonText: { color: '#fff', fontWeight: 'bold' },
});
```
