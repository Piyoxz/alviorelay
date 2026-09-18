/**
 * AlvioRelay WebRTC SFU — Web Client & Interactive Integration Showcase
 */

class AlvioDemoApp {
  constructor() {
    this.ws = null;
    this.localStream = null;
    this.screenStream = null;
    this.isAudioMuted = false;
    this.isVideoOff = false;
    this.isScreenSharing = false;
    this.isConnected = false;
    this.isInRoom = false;
    this.isDemoMode = false;
    this.peers = new Map();
    this.selfPeerId = 'local-peer-' + Math.random().toString(36).substring(2, 8);

    this.initDOMElements();
    this.bindEvents();
    this.initLocalMedia();
    this.startStatsTimer();
  }

  initDOMElements() {
    // Header & Status
    this.statusDot = document.getElementById('statusDot');
    this.statusLabel = document.getElementById('statusLabel');
    this.displayRoomName = document.getElementById('displayRoomName');
    this.peerCountText = document.getElementById('peerCountText');

    // Join Banner Inputs
    this.serverUrlInput = document.getElementById('serverUrlInput');
    this.roomIdInput = document.getElementById('roomIdInput');
    this.peerNameInput = document.getElementById('peerNameInput');
    this.joinRoomBtn = document.getElementById('joinRoomBtn');

    // Video Stage & Tiles
    this.videoGrid = document.getElementById('videoGrid');
    this.localTile = document.getElementById('localTile');
    this.localVideo = document.getElementById('localVideo');
    this.localAvatar = document.getElementById('localAvatar');
    this.localInitials = document.getElementById('localInitials');
    this.localTileName = document.getElementById('localTileName');
    this.localMicIndicator = document.getElementById('localMicIndicator');

    // Controls
    this.toggleMicBtn = document.getElementById('toggleMicBtn');
    this.toggleCamBtn = document.getElementById('toggleCamBtn');
    this.shareScreenBtn = document.getElementById('shareScreenBtn');
    this.toggleChatBtn = document.getElementById('toggleChatBtn');
    this.leaveBtn = document.getElementById('leaveBtn');

    // Chat Drawer
    this.chatDrawer = document.getElementById('chatDrawer');
    this.closeChatBtn = document.getElementById('closeChatBtn');
    this.chatMessages = document.getElementById('chatMessages');
    this.chatForm = document.getElementById('chatForm');
    this.chatInput = document.getElementById('chatInput');
    this.unreadDot = document.getElementById('unreadDot');

    // Stats HUD
    this.statsHud = document.getElementById('statsHud');
    this.openStatsBtn = document.getElementById('openStatsBtn');
    this.closeStatsHud = document.getElementById('closeStatsHud');
    this.statRtt = document.getElementById('statRtt');
    this.statBitrate = document.getElementById('statBitrate');
    this.statLoss = document.getElementById('statLoss');
    this.statCodec = document.getElementById('statCodec');

    // SDK Guide Modal
    this.openGuideBtn = document.getElementById('openGuideBtn');
    this.guideModalBackdrop = document.getElementById('guideModalBackdrop');
    this.closeGuideBtn = document.getElementById('closeGuideBtn');
    this.closeGuideFooterBtn = document.getElementById('closeGuideFooterBtn');
    this.tabButtons = document.querySelectorAll('.tab-btn');
    this.tabContents = document.querySelectorAll('.tab-content');
  }

  bindEvents() {
    this.joinRoomBtn.addEventListener('click', () => this.handleJoinRoom());
    this.toggleMicBtn.addEventListener('click', () => this.toggleMicrophone());
    this.toggleCamBtn.addEventListener('click', () => this.toggleCamera());
    this.shareScreenBtn.addEventListener('click', () => this.toggleScreenShare());
    this.toggleChatBtn.addEventListener('click', () => this.toggleChatDrawer());
    this.closeChatBtn.addEventListener('click', () => this.toggleChatDrawer(false));
    this.leaveBtn.addEventListener('click', () => this.leaveRoom());

    // Chat submit
    this.chatForm.addEventListener('submit', (e) => {
      e.preventDefault();
      this.sendChatMessage();
    });

    // Stats HUD toggles
    this.openStatsBtn.addEventListener('click', () => {
      this.statsHud.classList.toggle('hidden');
    });
    this.closeStatsHud.addEventListener('click', () => {
      this.statsHud.classList.add('hidden');
    });

    // Guide Modal
    this.openGuideBtn.addEventListener('click', () => {
      this.guideModalBackdrop.classList.remove('hidden');
    });
    this.closeGuideBtn.addEventListener('click', () => {
      this.guideModalBackdrop.classList.add('hidden');
    });
    this.closeGuideFooterBtn.addEventListener('click', () => {
      this.guideModalBackdrop.classList.add('hidden');
    });

    // Modal Tabs
    this.tabButtons.forEach((btn) => {
      btn.addEventListener('click', () => {
        const targetTab = btn.getAttribute('data-tab');
        this.tabButtons.forEach((b) => b.classList.remove('active'));
        this.tabContents.forEach((c) => c.classList.remove('active'));
        btn.classList.add('active');
        const content = document.getElementById(targetTab);
        if (content) content.classList.add('active');
      });
    });

    // Name input update initials
    this.peerNameInput.addEventListener('input', () => {
      this.updateUserInitials();
    });
  }

  updateUserInitials() {
    const name = this.peerNameInput.value.trim() || 'User';
    this.localTileName.textContent = name;
    const parts = name.split(' ');
    const initials = parts.length > 1
      ? (parts[0][0] + parts[1][0]).toUpperCase()
      : name.substring(0, 2).toUpperCase();
    this.localInitials.textContent = initials;
  }

  async initLocalMedia() {
    this.updateUserInitials();
    try {
      this.localStream = await navigator.mediaDevices.getUserMedia({
        audio: true,
        video: {
          width: { ideal: 1280 },
          height: { ideal: 720 },
          frameRate: { ideal: 30 }
        }
      });
      this.localVideo.srcObject = this.localStream;
      this.localTile.classList.remove('video-off');
    } catch (err) {
      console.warn('[AlvioApp] WebCam/Mic access not granted or unavailable:', err);
      this.localTile.classList.add('video-off');
      this.generateMockMediaCanvas();
    }
  }

  generateMockMediaCanvas() {
    const canvas = document.createElement('canvas');
    canvas.width = 640;
    canvas.height = 360;
    const ctx = canvas.getContext('2d');
    let hue = 180;

    const draw = () => {
      hue = (hue + 0.5) % 360;
      ctx.fillStyle = `hsl(${hue}, 40%, 12%)`;
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      ctx.fillStyle = '#00e5ff';
      ctx.font = 'bold 24px Outfit, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText(this.peerNameInput.value || 'Local User', canvas.width / 2, canvas.height / 2 - 10);

      ctx.fillStyle = '#94a3b8';
      ctx.font = '14px Inter, sans-serif';
      ctx.fillText('AlvioRelay WebRTC Local Stream', canvas.width / 2, canvas.height / 2 + 25);

      requestAnimationFrame(draw);
    };
    draw();

    this.localStream = canvas.captureStream(30);
    this.localVideo.srcObject = this.localStream;
    this.localTile.classList.remove('video-off');
  }

  toggleMicrophone() {
    this.isAudioMuted = !this.isAudioMuted;
    if (this.localStream) {
      this.localStream.getAudioTracks().forEach((track) => {
        track.enabled = !this.isAudioMuted;
      });
    }

    const iconOn = this.toggleMicBtn.querySelector('.icon-on');
    const iconOff = this.toggleMicBtn.querySelector('.icon-off');

    if (this.isAudioMuted) {
      this.toggleMicBtn.classList.add('active-off');
      iconOn.classList.add('hidden');
      iconOff.classList.remove('hidden');
      this.localMicIndicator.classList.remove('active');
      this.localMicIndicator.classList.add('muted');
    } else {
      this.toggleMicBtn.classList.remove('active-off');
      iconOn.classList.remove('hidden');
      iconOff.classList.add('hidden');
      this.localMicIndicator.classList.add('active');
      this.localMicIndicator.classList.remove('muted');
    }
  }

  toggleCamera() {
    this.isVideoOff = !this.isVideoOff;
    if (this.localStream) {
      this.localStream.getVideoTracks().forEach((track) => {
        track.enabled = !this.isVideoOff;
      });
    }

    const iconOn = this.toggleCamBtn.querySelector('.icon-on');
    const iconOff = this.toggleCamBtn.querySelector('.icon-off');

    if (this.isVideoOff) {
      this.toggleCamBtn.classList.add('active-off');
      iconOn.classList.add('hidden');
      iconOff.classList.remove('hidden');
      this.localTile.classList.add('video-off');
    } else {
      this.toggleCamBtn.classList.remove('active-off');
      iconOn.classList.remove('hidden');
      iconOff.classList.add('hidden');
      this.localTile.classList.remove('video-off');
    }
  }

  async toggleScreenShare() {
    if (this.isScreenSharing) {
      if (this.screenStream) {
        this.screenStream.getTracks().forEach((t) => t.stop());
      }
      this.localVideo.srcObject = this.localStream;
      this.isScreenSharing = false;
      this.shareScreenBtn.classList.remove('active-accent');
      return;
    }

    try {
      this.screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: { cursor: 'always' },
        audio: false,
      });
      this.localVideo.srcObject = this.screenStream;
      this.isScreenSharing = true;
      this.shareScreenBtn.classList.add('active-accent');

      this.screenStream.getVideoTracks()[0].onended = () => {
        this.toggleScreenShare();
      };
    } catch (err) {
      console.warn('[AlvioApp] Screen share cancelled:', err);
    }
  }

  toggleChatDrawer(forceState) {
    const isHidden = this.chatDrawer.classList.contains('hidden');
    const shouldOpen = forceState !== undefined ? forceState : isHidden;

    if (shouldOpen) {
      this.chatDrawer.classList.remove('hidden');
      this.toggleChatBtn.classList.add('active-accent');
      this.unreadDot.classList.add('hidden');
      this.chatInput.focus();
    } else {
      this.chatDrawer.classList.add('hidden');
      this.toggleChatBtn.classList.remove('active-accent');
    }
  }

  async handleJoinRoom() {
    const url = this.serverUrlInput.value.trim();
    const roomId = this.roomIdInput.value.trim() || 'ruang-utama';
    const peerName = this.peerNameInput.value.trim() || 'Budi Developer';

    this.displayRoomName.textContent = roomId;
    this.setStatus('connecting', 'Menghubungkan...');

    try {
      await this.connectSignaling(url, roomId, peerName);
    } catch (err) {
      console.warn('[AlvioApp] Tidak dapat terhubung ke real backend. Masuk ke Mode Demonstrasi Interaktif:', err);
      this.enterInteractiveDemoMode(roomId, peerName);
    }
  }

  connectSignaling(url, roomId, peerName) {
    return new Promise((resolve, reject) => {
      let resolved = false;

      try {
        this.ws = new WebSocket(url);
      } catch (e) {
        return reject(e);
      }

      const connectionTimeout = setTimeout(() => {
        if (!resolved) {
          if (this.ws) this.ws.close();
          reject(new Error('Connection timed out'));
        }
      }, 2500);

      this.ws.onopen = () => {
        clearTimeout(connectionTimeout);
        resolved = true;
        this.isConnected = true;
        this.setStatus('connected', 'Terhubung ke Gateway');

        // Send Connect
        this.sendSignal('connect', {
          client_version: 'alvio-web-demo-0.1.0',
        });

        // Send Join
        this.sendSignal('join', {
          room_id: roomId,
          peer_name: peerName,
        });

        this.isInRoom = true;
        this.setStatus('connected', `Aktif: ${roomId}`);
        document.getElementById('joinBanner').classList.add('hidden');
        resolve();
      };

      this.ws.onmessage = (event) => {
        try {
          const envelope = JSON.parse(event.data);
          this.handleSignalingMessage(envelope);
        } catch (e) {
          console.error('[Signaling] Parse error:', e);
        }
      };

      this.ws.onerror = (err) => {
        if (!resolved) {
          clearTimeout(connectionTimeout);
          reject(err);
        }
      };

      this.ws.onclose = () => {
        this.setStatus('disconnected', 'Terputus');
        this.isConnected = false;
        this.isInRoom = false;
      };
    });
  }

  handleSignalingMessage(envelope) {
    switch (envelope.type) {
      case 'ack':
        this.selfPeerId = envelope.payload?.peer_id || this.selfPeerId;
        break;
      case 'room_joined':
        for (const p of envelope.payload?.peers || []) {
          this.addRemotePeer(p.id, p.name);
        }
        break;
      case 'peer_joined':
        if (envelope.payload?.peer) {
          this.addRemotePeer(envelope.payload.peer.id, envelope.payload.peer.name);
          this.appendChatMessage('system', `${envelope.payload.peer.name} bergabung ke ruangan`);
        }
        break;
      case 'peer_left':
        this.removeRemotePeer(envelope.payload?.peer_id);
        break;
      case 'data_received':
        this.appendChatMessage('remote', envelope.payload?.payload, envelope.payload?.source_peer_id);
        break;
    }
  }

  sendSignal(type, payload) {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      const envelope = {
        version: 1,
        id: `msg_${Math.random().toString(36).substring(2, 9)}`,
        type,
        payload,
      };
      this.ws.send(JSON.stringify(envelope));
    }
  }

  enterInteractiveDemoMode(roomId, peerName) {
    this.isDemoMode = true;
    this.isInRoom = true;
    this.setStatus('connected', `Demo Mode: ${roomId}`);
    document.getElementById('joinBanner').classList.add('hidden');

    this.appendChatMessage('system', `Anda memasuki demo interaktif AlvioRelay SFU di ruangan "${roomId}".`);

    // Add Simulated Remote Participants
    setTimeout(() => {
      this.addRemotePeer('peer-sarah-101', 'Sarah Tan (Singapore)', 'high');
      this.appendChatMessage('remote', 'Halo semuanya! Audio & video lancar via Alvio SFU.', 'Sarah Tan');
    }, 600);

    setTimeout(() => {
      this.addRemotePeer('peer-kenji-202', 'Kenji Sato (Tokyo)', 'medium');
    }, 1400);

    setTimeout(() => {
      this.addRemotePeer('peer-alvio-bot', 'Alvio AI Assistant', 'high');
      this.appendChatMessage('remote', 'Latency RTT saat ini: 12ms. Zero frame drop terdeteksi.', 'Alvio AI Assistant');
    }, 2200);
  }

  addRemotePeer(peerId, name, initialLayer = 'high') {
    if (this.peers.has(peerId)) return;

    this.peers.set(peerId, { id: peerId, name, layer: initialLayer });

    const tile = document.createElement('div');
    tile.className = 'video-tile';
    tile.id = `tile-${peerId}`;

    const initials = name.split(' ').map((n) => n[0]).join('').substring(0, 2).toUpperCase();

    tile.innerHTML = `
      <div class="video-wrapper">
        <canvas id="canvas-${peerId}" width="640" height="360"></canvas>
        <div class="tile-watermark">Alvio Simulcast Stream</div>
      </div>
      <div class="simulcast-controls">
        <button class="simulcast-btn ${initialLayer === 'high' ? 'active' : ''}" data-layer="high" title="720p HD">High</button>
        <button class="simulcast-btn ${initialLayer === 'medium' ? 'active' : ''}" data-layer="medium" title="360p SD">Med</button>
        <button class="simulcast-btn ${initialLayer === 'low' ? 'active' : ''}" data-layer="low" title="180p Low Bandwidth">Low</button>
      </div>
      <div class="tile-overlay">
        <div class="tile-info">
          <span class="peer-name">${name}</span>
        </div>
        <div class="tile-indicators">
          <div class="mic-status active" title="Mikrofon Aktif">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" y1="19" x2="12" y2="22"/>
            </svg>
          </div>
          <div class="layer-badge" id="badge-${peerId}">${initialLayer.toUpperCase()} 720p</div>
        </div>
      </div>
    `;

    this.videoGrid.appendChild(tile);
    this.startPeerCanvasAnimation(`canvas-${peerId}`, name, initials);

    // Bind simulcast buttons
    const btns = tile.querySelectorAll('.simulcast-btn');
    btns.forEach((btn) => {
      btn.addEventListener('click', () => {
        const layer = btn.getAttribute('data-layer');
        btns.forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
        this.switchPeerLayer(peerId, layer);
      });
    });

    this.updatePeerCounter();
  }

  switchPeerLayer(peerId, layer) {
    const peer = this.peers.get(peerId);
    if (!peer) return;

    peer.layer = layer;
    const badge = document.getElementById(`badge-${peerId}`);
    if (badge) {
      const resText = layer === 'high' ? '720p' : layer === 'medium' ? '360p' : '180p';
      badge.textContent = `${layer.toUpperCase()} ${resText}`;
    }

    if (!this.isDemoMode) {
      this.sendSignal('layer_select', {
        track_id: `trk_${peerId}`,
        layer: layer,
      });
    } else {
      console.log(`[Simulcast] Dynamic layer switched for ${peer.name} to ${layer}`);
    }
  }

  removeRemotePeer(peerId) {
    const tile = document.getElementById(`tile-${peerId}`);
    if (tile) tile.remove();
    this.peers.delete(peerId);
    this.updatePeerCounter();
  }

  updatePeerCounter() {
    const total = this.peers.size + 1;
    this.peerCountText.textContent = `${total} Peserta`;
  }

  startPeerCanvasAnimation(canvasId, name, initials) {
    const canvas = document.getElementById(canvasId);
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    let phase = Math.random() * 100;

    const render = () => {
      phase += 0.02;
      ctx.fillStyle = '#0c1220';
      ctx.fillRect(0, 0, canvas.width, canvas.height);

      // Draw subtle wave
      ctx.strokeStyle = 'rgba(0, 229, 255, 0.15)';
      ctx.lineWidth = 2;
      ctx.beginPath();
      for (let x = 0; x < canvas.width; x += 10) {
        const y = canvas.height / 2 + Math.sin(x * 0.015 + phase) * 25;
        if (x === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
      }
      ctx.stroke();

      // Draw Avatar Circle
      ctx.fillStyle = 'rgba(22, 31, 51, 0.8)';
      ctx.beginPath();
      ctx.arc(canvas.width / 2, canvas.height / 2 - 15, 45, 0, Math.PI * 2);
      ctx.fill();
      ctx.strokeStyle = 'rgba(0, 229, 255, 0.4)';
      ctx.stroke();

      // Draw Initials
      ctx.fillStyle = '#f1f5f9';
      ctx.font = 'bold 24px Outfit, sans-serif';
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';
      ctx.fillText(initials, canvas.width / 2, canvas.height / 2 - 15);

      // Draw Name
      ctx.fillStyle = '#cbd5e1';
      ctx.font = '14px Inter, sans-serif';
      ctx.fillText(name, canvas.width / 2, canvas.height / 2 + 50);

      requestAnimationFrame(render);
    };
    render();
  }

  sendChatMessage() {
    const text = this.chatInput.value.trim();
    if (!text) return;

    this.appendChatMessage('you', text);
    this.chatInput.value = '';

    if (!this.isDemoMode) {
      this.sendSignal('data_message', {
        destination_peer_ids: [],
        payload: text,
        reliable: true,
      });
    } else {
      // Echo response in demo mode
      setTimeout(() => {
        const replies = [
          'Pesan diterima melalui SCTP Data Channel!',
          'Streaming lancar, latency stabil sub-15ms.',
          'Mantap, layer simulcast adaptif berjalan mulus!',
        ];
        const randomReply = replies[Math.floor(Math.random() * replies.length)];
        this.appendChatMessage('remote', randomReply, 'Sarah Tan');
      }, 1000);
    }
  }

  appendChatMessage(type, text, senderName) {
    const msgDiv = document.createElement('div');
    msgDiv.className = `chat-msg ${type}`;

    if (type === 'system') {
      msgDiv.innerHTML = `<span class="system-text">${text}</span>`;
    } else if (type === 'you') {
      msgDiv.innerHTML = `
        <span class="msg-sender">Anda</span>
        <div class="msg-bubble">${this.escapeHTML(text)}</div>
      `;
    } else {
      msgDiv.innerHTML = `
        <span class="msg-sender">${this.escapeHTML(senderName || 'Peserta')}</span>
        <div class="msg-bubble">${this.escapeHTML(text)}</div>
      `;
      // Show unread indicator if drawer is closed
      if (this.chatDrawer.classList.contains('hidden')) {
        this.unreadDot.classList.remove('hidden');
      }
    }

    this.chatMessages.appendChild(msgDiv);
    this.chatMessages.scrollTop = this.chatMessages.scrollHeight;
  }

  escapeHTML(str) {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  setStatus(state, label) {
    this.statusDot.className = `pulse-indicator ${state}`;
    this.statusLabel.textContent = label;
  }

  startStatsTimer() {
    setInterval(() => {
      if (this.isInRoom) {
        const jitter = (Math.random() * 4 - 2).toFixed(1);
        const rtt = Math.max(9, (13 + parseFloat(jitter))).toFixed(0);
        const bitrate = (2.2 + Math.random() * 0.4).toFixed(1);

        this.statRtt.textContent = `${rtt} ms`;
        this.statBitrate.textContent = `${bitrate} Mbps`;
        this.statLoss.textContent = '0.0 %';
      }
    }, 2000);
  }

  leaveRoom() {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
    this.peers.clear();
    const remoteTiles = this.videoGrid.querySelectorAll('.video-tile:not(.local-tile)');
    remoteTiles.forEach((t) => t.remove());

    this.isInRoom = false;
    this.isDemoMode = false;
    this.setStatus('disconnected', 'Standby');
    this.updatePeerCounter();
    document.getElementById('joinBanner').classList.remove('hidden');
    this.appendChatMessage('system', 'Anda telah meninggalkan ruangan.');
  }
}

// Inisialisasi saat DOM siap
window.addEventListener('DOMContentLoaded', () => {
  window.alvioApp = new AlvioDemoApp();
});
