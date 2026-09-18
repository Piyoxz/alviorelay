# AlvioRelay React Native SDK (`@alviorelay/react-native`)

High-performance WebRTC client library for React Native applications on iOS and Android.

## Installation

```bash
npm install @alviorelay/react-native @alviorelay/client react-native-webrtc
# or
yarn add @alviorelay/react-native @alviorelay/client react-native-webrtc
```

## Quickstart

```tsx
import React, { useEffect, useState } from 'react';
import { View, Text, Button } from 'react-native';
import { AlvioRoom } from '@alviorelay/client';
import { RTCView } from 'react-native-webrtc';

export function CallScreen() {
  const [room] = useState(() => new AlvioRoom());
  const [status, setStatus] = useState('Disconnected');

  useEffect(() => {
    room.on('connected', () => setStatus('Connected to Alvio Relay'));
    room.on('roomJoined', (data) => console.log('Joined room:', data.roomId));

    room.connect('wss://relay.alvio.io/ws');

    return () => {
      room.disconnect();
    };
  }, [room]);

  return (
    <View style={{ flex: 1, justifyContent: 'center', alignItems: 'center' }}>
      <Text>Status: {status}</Text>
      <Button
        title="Join Room"
        onPress={() => room.join('mobile-room', 'RN User')}
      />
    </View>
  );
}
```
