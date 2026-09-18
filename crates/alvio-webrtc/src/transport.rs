use crate::packet::AlvioRtpPacket;
use alvio_core::{AlvioError, AlvioResult};
use bytes::Bytes;
use std::net::SocketAddr;
use std::time::Instant;
use str0m::change::SdpOffer;
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, Input, Output, Rtc};
use tracing::{debug, info, warn};

pub use str0m::channel::{ChannelId, Reliability};

/// Data packet carried across a WebRTC SCTP DataChannel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlvioDataPacket {
    pub channel_id: ChannelId,
    pub label: String,
    pub binary: bool,
    pub data: Bytes,
}

#[derive(Debug)]
pub enum AlvioTransportOutput {
    /// UDP packet that must be transmitted over the socket to a destination address.
    Transmit {
        destination: SocketAddr,
        contents: Vec<u8>,
    },
    /// An incoming media RTP packet received from the peer.
    Rtp(AlvioRtpPacket),
    /// Peer has completed DTLS and ICE handshake and is connected.
    Connected,
    /// Connection state changed to disconnected.
    Disconnected,
    /// Downstream peer requested an intra keyframe (PLI/FIR).
    KeyframeRequest {
        mid: String,
        kind: str0m::media::KeyframeRequestKind,
    },
    /// Egress Bandwidth estimate update (from TWCC / GCC) in bps.
    EgressBitrateEstimate(u64),
    /// WebRTC DataChannel opened.
    DataChannelOpen {
        channel_id: ChannelId,
        label: String,
    },
    /// Incoming data packet from remote peer on a DataChannel.
    DataChannelData(AlvioDataPacket),
    /// DataChannel closed.
    DataChannelClose { channel_id: ChannelId },
    /// Next deadline when `handle_timeout` should be invoked.
    Timeout(Instant),
}

/// AlvioRelay WebRTC Transport wrapping the Sans-I/O `str0m` engine.
pub struct AlvioTransport {
    rtc: Rtc,
    local_addr: SocketAddr,
    connected: bool,
    channel_labels: std::collections::HashMap<ChannelId, String>,
}

impl AlvioTransport {
    pub fn new(local_addr: SocketAddr) -> AlvioResult<Self> {
        let mut rtc = Rtc::builder().set_rtp_mode(true).build(Instant::now());

        let candidate = Candidate::host(local_addr, "udp")
            .map_err(|e| AlvioError::Transport(format!("Failed to create host candidate: {e}")))?;
        rtc.add_local_candidate(candidate);

        Ok(Self {
            rtc,
            local_addr,
            connected: false,
            channel_labels: std::collections::HashMap::new(),
        })
    }

    /// Accepts an incoming SDP Offer string from a remote peer and returns an SDP Answer.
    pub fn accept_remote_offer(&mut self, sdp: &str) -> AlvioResult<String> {
        let offer = SdpOffer::from_sdp_string(sdp)
            .map_err(|e| AlvioError::Transport(format!("Failed to parse remote SDP offer: {e}")))?;

        let answer = self
            .rtc
            .sdp_api()
            .accept_offer(offer)
            .map_err(|e| AlvioError::Transport(format!("Failed to accept SDP offer: {e}")))?;

        let answer_sdp = answer.to_sdp_string();
        info!(
            "Accepted remote SDP offer, generated SDP answer (len={})",
            answer_sdp.len()
        );
        Ok(answer_sdp)
    }

    /// Creates an outgoing DataChannel with the given label and reliability configuration.
    pub fn create_data_channel(
        &mut self,
        label: &str,
        ordered: bool,
        max_retransmits: Option<u16>,
    ) -> ChannelId {
        let reliability = if let Some(n) = max_retransmits {
            Reliability::MaxRetransmits { retransmits: n }
        } else {
            Reliability::Reliable
        };

        let config = str0m::channel::ChannelConfig {
            label: label.to_string(),
            ordered,
            reliability,
            ..Default::default()
        };

        let id = self.rtc.direct_api().create_data_channel(config);
        self.channel_labels.insert(id, label.to_string());
        id
    }

    /// Transmits data on an active DataChannel.
    pub fn send_data_channel(
        &mut self,
        channel_id: ChannelId,
        binary: bool,
        data: &[u8],
    ) -> AlvioResult<bool> {
        if let Some(mut chan) = self.rtc.channel(channel_id) {
            chan.write(binary, data)
                .map_err(|e| AlvioError::Transport(format!("Failed to write to DataChannel: {e}")))
        } else {
            Ok(false)
        }
    }

    /// Feed an incoming UDP packet into the Sans-I/O WebRTC state machine.
    pub fn handle_input(
        &mut self,
        now: Instant,
        source: SocketAddr,
        data: &[u8],
    ) -> AlvioResult<()> {
        let receive = Receive {
            proto: Protocol::Udp,
            source,
            destination: self.local_addr,
            contents: data.try_into().map_err(|_| {
                AlvioError::Transport("UDP packet exceeds max MTU size".to_string())
            })?,
        };

        self.rtc
            .handle_input(Input::Receive(now, receive))
            .map_err(|e| AlvioError::Transport(format!("WebRTC handle_input error: {e}")))?;

        Ok(())
    }

    /// Advance internal engine time to deadline.
    pub fn handle_timeout(&mut self, now: Instant) -> AlvioResult<()> {
        self.rtc
            .handle_input(Input::Timeout(now))
            .map_err(|e| AlvioError::Transport(format!("WebRTC handle_timeout error: {e}")))?;
        Ok(())
    }

    /// Polls for the next output from the Sans-I/O state machine.
    pub fn poll_output(&mut self) -> AlvioResult<Option<AlvioTransportOutput>> {
        match self.rtc.poll_output() {
            Ok(Output::Transmit(t)) => Ok(Some(AlvioTransportOutput::Transmit {
                destination: t.destination,
                contents: t.contents.to_vec(),
            })),
            Ok(Output::Timeout(t)) => Ok(Some(AlvioTransportOutput::Timeout(t))),
            Ok(Output::Event(event)) => match event {
                Event::Connected => {
                    self.connected = true;
                    info!("WebRTC Transport DTLS-SRTP handshake completed (Connected)");
                    Ok(Some(AlvioTransportOutput::Connected))
                }
                Event::IceConnectionStateChange(state) => {
                    debug!("ICE connection state changed: {:?}", state);
                    if state.is_disconnected() {
                        self.connected = false;
                        Ok(Some(AlvioTransportOutput::Disconnected))
                    } else {
                        Ok(None)
                    }
                }
                Event::KeyframeRequest(req) => {
                    debug!(mid = %req.mid, kind = ?req.kind, "Received downstream KeyframeRequest");
                    Ok(Some(AlvioTransportOutput::KeyframeRequest {
                        mid: req.mid.to_string(),
                        kind: req.kind,
                    }))
                }
                Event::EgressBitrateEstimate(bwe) => {
                    let bps = match bwe {
                        str0m::bwe::BweKind::Twcc(rate) => rate.as_u64(),
                        str0m::bwe::BweKind::Remb(_, rate) => rate.as_u64(),
                        _ => 0,
                    };
                    debug!(bitrate_bps = bps, "Received EgressBitrateEstimate");
                    Ok(Some(AlvioTransportOutput::EgressBitrateEstimate(bps)))
                }
                Event::ChannelOpen(id, label) => {
                    debug!(?id, %label, "WebRTC DataChannel opened");
                    self.channel_labels.insert(id, label.clone());
                    Ok(Some(AlvioTransportOutput::DataChannelOpen {
                        channel_id: id,
                        label,
                    }))
                }
                Event::ChannelData(data) => {
                    let label = self
                        .channel_labels
                        .get(&data.id)
                        .cloned()
                        .unwrap_or_default();
                    let packet = AlvioDataPacket {
                        channel_id: data.id,
                        label,
                        binary: data.binary,
                        data: Bytes::from(data.data),
                    };
                    Ok(Some(AlvioTransportOutput::DataChannelData(packet)))
                }
                Event::ChannelClose(id) => {
                    debug!(?id, "WebRTC DataChannel closed");
                    self.channel_labels.remove(&id);
                    Ok(Some(AlvioTransportOutput::DataChannelClose {
                        channel_id: id,
                    }))
                }
                Event::RtpPacket(rtp) => {
                    let packet = AlvioRtpPacket {
                        header: crate::packet::RtpHeader {
                            version: rtp.header.version,
                            has_padding: rtp.header.has_padding,
                            has_extension: rtp.header.has_extension,
                            csrc_count: rtp.header.csrc_count as u8,
                            marker: rtp.header.marker,
                            payload_type: *rtp.header.payload_type,
                            sequence_number: rtp.header.sequence_number,
                            timestamp: rtp.header.timestamp,
                            ssrc: *rtp.header.ssrc,
                        },
                        payload: Bytes::copy_from_slice(&rtp.payload),
                    };
                    Ok(Some(AlvioTransportOutput::Rtp(packet)))
                }
                _ => Ok(None),
            },
            Err(e) => {
                warn!("WebRTC poll_output error: {:?}", e);
                Err(AlvioError::Transport(format!(
                    "WebRTC poll_output error: {e}"
                )))
            }
        }
    }

    /// Dispatches a keyframe request (PLI or FIR) upstream to the publisher of the given SSRC.
    pub fn request_keyframe(&mut self, ssrc: u32, kind: str0m::media::KeyframeRequestKind) -> bool {
        let ssrc_val = str0m::rtp::Ssrc::from(ssrc);
        if let Some(stream) = self.rtc.direct_api().stream_rx(&ssrc_val) {
            stream.request_keyframe(kind);
            true
        } else {
            false
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}
