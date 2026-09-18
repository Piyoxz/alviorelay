# Panduan Integrasi Client SDK (Web) — `@alviorelay/client`

Dokumentasi resmi integrasi client web AlvioRelay untuk browser modern (Chrome, Edge, Firefox, Safari) menggunakan TypeScript / JavaScript.

---

## 1. Instalasi

Gunakan npm, pnpm, atau yarn untuk menginstal SDK:

```bash
npm install @alviorelay/client
# atau
yarn add @alviorelay/client
# atau
pnpm add @alviorelay/client
```

---

## 2. Inisialisasi & Koneksi Ruangan

Berikut adalah pola standar untuk mengkoneksikan client web ke server signaling AlvioRelay:

```typescript
import { AlvioRoom, StreamLayer } from '@alviorelay/client';

// 1. Instansiasi objek AlvioRoom
const room = new AlvioRoom();

// 2. Daftarkan event listener
room.on('connected', ({ peerId, nodeId }) => {
  console.log(`Terhubung ke AlvioRelay node ${nodeId} dengan Peer ID: ${peerId}`);
  // Langsung gabung ke ruangan yang diinginkan
  room.join('ruang-rapat-utama', 'Budi Pratama');
});

room.on('roomJoined', ({ roomId, peers, tracks }) => {
  console.log(`Berhasil bergabung ke ruangan ${roomId}`);
  console.log(`Daftar peserta yang sudah ada:`, peers);
  console.log(`Daftar track aktif:`, tracks);
});

room.on('peerJoined', (peer) => {
  console.log(`Peserta baru bergabung: ${peer.name} (${peer.id})`);
});

room.on('peerLeft', ({ peerId, reason }) => {
  console.log(`Peserta keluar: ${peerId}, alasan: ${reason}`);
});

room.on('trackPublished', (track) => {
  console.log(`Track baru tersedia: ${track.kind} (${track.id}) dari ${track.peer_id}`);
  // Otomatis subscribe ke track tersebut
  room.subscribe(track.id, { preferredLayer: 'high' });
});

room.on('dataReceived', ({ sourcePeerId, payload }) => {
  console.log(`Pesan obrolan dari ${sourcePeerId}:`, payload);
});

// 3. Eksekusi koneksi ke WebSocket Signaling Gateway
await room.connect('wss://relay.perusahaan.com/ws', 'JWT_AUTH_TOKEN_JIKA_ADA');
```

---

## 3. Publikasi Media (Kamera & Mikrofon)

Gunakan API browser native `navigator.mediaDevices.getUserMedia` untuk mendapatkan media track, lalu publikasikan melalui AlvioRoom:

```typescript
// Dapatkan stream lokal dari kamera dan mikrofon
const localStream = await navigator.mediaDevices.getUserMedia({
  audio: {
    echoCancellation: true,
    noiseSuppression: true,
    autoGainControl: true,
  },
  video: {
    width: { ideal: 1280 },
    height: { ideal: 720 },
    frameRate: { ideal: 30 },
  },
});

// Publikasikan track mikrofon (Audio)
const audioTrack = localStream.getAudioTracks()[0];
if (audioTrack) {
  await room.publishTrack(audioTrack, { source: 'microphone' });
}

// Publikasikan track kamera (Video) dengan Simulcast aktif
const videoTrack = localStream.getVideoTracks()[0];
if (videoTrack) {
  await room.publishTrack(videoTrack, {
    source: 'camera',
    simulcast: true, // Mengaktifkan layer High (720p), Medium (360p), Low (180p)
    layers: ['low', 'medium', 'high'],
  });
}
```

---

## 4. Screen Sharing (Bagi Layar)

Untuk berbagi presentasi atau layar desktop:

```typescript
const displayStream = await navigator.mediaDevices.getDisplayMedia({
  video: {
    cursor: 'always',
    frameRate: { max: 30 },
  },
  audio: false,
});

const screenTrack = displayStream.getVideoTracks()[0];
await room.publishTrack(screenTrack, {
  source: 'screen_share',
  simulcast: false, // Screen sharing umumnya tidak memerlukan simulcast agresif
  layers: ['high'],
});

// Bersihkan track saat pengguna menekan tombol "Stop Sharing" bawaan browser
screenTrack.onended = async () => {
  console.log('Screen share dihentikan oleh user');
};
```

---

## 5. Pemilihan Layer Simulcast Adaptif (Dynamic Layer Selection)

Ketika koneksi klien mengalami degradasi bandwidth atau perangkat berada di tampilan grid kecil (*thumbnail*), klien dapat beralih ke layer rendah secara dinamis:

```typescript
// Beralih ke kualitas rendah (180p / 150 kbps) untuk menghemat kuota / CPU
await room.setPreferredLayer('trk_video_12345', 'low');

// Beralih kembali ke kualitas tinggi (720p / 1.5 Mbps) saat user membuka tampilan layar penuh (Pin/Spotlight)
await room.setPreferredLayer('trk_video_12345', 'high');
```

---

## 6. Mengirim Pesan Data Channel (Chat, Reaksi, Telemetri)

AlvioRelay mendukung pengiriman data ultra-rendah latensi:

```typescript
// Mengirim pesan obrolan broadcast ke seluruh peserta ruangan (Reliable SCTP)
await room.sendData(JSON.stringify({ type: 'chat', text: 'Halo semuanya!' }), {
  reliable: true,
});

// Mengirim posisi kursor mouse whiteboard (Unreliable lossy untuk latensi sub-10ms)
await room.sendData(JSON.stringify({ type: 'cursor', x: 420, y: 180 }), {
  reliable: false,
});
```

---

## 7. Keluar dan Pembersihan Resource

```typescript
// Keluar dari ruangan saat ini namun tetap terhubung ke gateway
await room.leave();

// Atau putuskan koneksi sepenuhnya saat menutup aplikasi
room.disconnect();
```
