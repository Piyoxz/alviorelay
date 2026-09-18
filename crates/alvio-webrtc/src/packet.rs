use bytes::{BufMut, Bytes, BytesMut};
use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RtpParseError {
    #[error("Packet length is too short to contain a valid RTP header")]
    TooShort,
    #[error("Unsupported RTP version: {0} (expected 2)")]
    UnsupportedVersion(u8),
}

/// Standard 12-byte RTP fixed header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtpHeader {
    pub version: u8,
    pub has_padding: bool,
    pub has_extension: bool,
    pub csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
}

impl RtpHeader {
    pub const FIXED_SIZE: usize = 12;

    pub fn parse(buf: &[u8]) -> Result<(Self, usize), RtpParseError> {
        if buf.len() < Self::FIXED_SIZE {
            return Err(RtpParseError::TooShort);
        }

        let b0 = buf[0];
        let version = b0 >> 6;
        if version != 2 {
            return Err(RtpParseError::UnsupportedVersion(version));
        }

        let has_padding = (b0 & 0x20) != 0;
        let has_extension = (b0 & 0x10) != 0;
        let csrc_count = b0 & 0x0F;

        let b1 = buf[1];
        let marker = (b1 & 0x80) != 0;
        let payload_type = b1 & 0x7F;

        let sequence_number = u16::from_be_bytes([buf[2], buf[3]]);
        let timestamp = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let ssrc = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);

        let header_len = Self::FIXED_SIZE + (csrc_count as usize * 4);
        if buf.len() < header_len {
            return Err(RtpParseError::TooShort);
        }

        Ok((
            Self {
                version,
                has_padding,
                has_extension,
                csrc_count,
                marker,
                payload_type,
                sequence_number,
                timestamp,
                ssrc,
            },
            header_len,
        ))
    }

    pub fn write_into(&self, buf: &mut [u8]) {
        let b0 = (self.version << 6)
            | ((self.has_padding as u8) << 5)
            | ((self.has_extension as u8) << 4)
            | (self.csrc_count & 0x0F);
        let b1 = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);

        buf[0] = b0;
        buf[1] = b1;
        buf[2..4].copy_from_slice(&self.sequence_number.to_be_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());
    }
}

/// Zero-allocation, ref-counted RTP packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlvioRtpPacket {
    pub header: RtpHeader,
    pub payload: Bytes,
}

impl AlvioRtpPacket {
    pub fn parse(bytes: Bytes) -> Result<Self, RtpParseError> {
        let (header, header_len) = RtpHeader::parse(&bytes)?;
        let payload = bytes.slice(header_len..);
        Ok(Self { header, payload })
    }

    pub fn with_rewritten_meta(&self, ssrc: u32, sequence_number: u16, timestamp: u32) -> Self {
        let mut new_header = self.header;
        new_header.ssrc = ssrc;
        new_header.sequence_number = sequence_number;
        new_header.timestamp = timestamp;
        Self {
            header: new_header,
            payload: self.payload.clone(),
        }
    }

    pub fn serialize(&self) -> Bytes {
        let mut buf = BytesMut::with_capacity(RtpHeader::FIXED_SIZE + self.payload.len());
        let mut header_bytes = [0u8; RtpHeader::FIXED_SIZE];
        self.header.write_into(&mut header_bytes);
        buf.put_slice(&header_bytes);
        buf.put_slice(&self.payload);
        buf.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_header_parse_and_write() {
        let header = RtpHeader {
            version: 2,
            has_padding: false,
            has_extension: false,
            csrc_count: 0,
            marker: true,
            payload_type: 96, // VP8
            sequence_number: 1042,
            timestamp: 90000,
            ssrc: 0x12345678,
        };

        let mut raw = [0u8; 12];
        header.write_into(&mut raw);

        let (parsed, len) = RtpHeader::parse(&raw).unwrap();
        assert_eq!(len, 12);
        assert_eq!(parsed, header);
    }

    #[test]
    fn test_packet_rewriting() {
        let mut raw = vec![
            0x80, 0x60, 0x00, 0x05, // v2, pt 96, seq 5
            0x00, 0x01, 0x5F, 0x90, // ts 90000
            0x11, 0x22, 0x33, 0x44, // ssrc
        ];
        raw.extend_from_slice(b"sample_vp8_frame_payload");

        let packet = AlvioRtpPacket::parse(Bytes::from(raw)).unwrap();
        assert_eq!(packet.header.sequence_number, 5);
        assert_eq!(packet.header.ssrc, 0x11223344);
        assert_eq!(packet.payload.as_ref(), b"sample_vp8_frame_payload");

        // Rewrite SSRC to 0xAABBCCDD and sequence number to 100
        let rewritten = packet.with_rewritten_meta(0xAABBCCDD, 100, 95000);
        assert_eq!(rewritten.header.ssrc, 0xAABBCCDD);
        assert_eq!(rewritten.header.sequence_number, 100);
        assert_eq!(rewritten.header.timestamp, 95000);
        assert_eq!(rewritten.payload.as_ref(), b"sample_vp8_frame_payload");

        let serialized = rewritten.serialize();
        let re_parsed = AlvioRtpPacket::parse(serialized).unwrap();
        assert_eq!(re_parsed.header.ssrc, 0xAABBCCDD);
        assert_eq!(re_parsed.header.sequence_number, 100);
    }
}
