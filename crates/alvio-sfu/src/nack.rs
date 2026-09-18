use alvio_webrtc::AlvioRtpPacket;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Fixed-size circular ring buffer for caching transmitted/received RTP packets.
///
/// Enables local retransmission of lost packets without fetching from upstream publisher (<1 RTT).
pub struct NackBuffer {
    capacity: usize,
    mask: usize,
    buffer: RwLock<Vec<Option<AlvioRtpPacket>>>,
}

impl NackBuffer {
    pub const DEFAULT_CAPACITY: usize = 512;

    /// Creates a new NackBuffer with the given capacity (rounded up to power of two).
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.next_power_of_two().max(64);
        let mut vec = Vec::with_capacity(cap);
        for _ in 0..cap {
            vec.push(None);
        }

        Self {
            capacity: cap,
            mask: cap - 1,
            buffer: RwLock::new(vec),
        }
    }

    /// Store a packet in the circular buffer.
    pub fn put(&self, packet: AlvioRtpPacket) {
        let idx = (packet.header.sequence_number as usize) & self.mask;
        let mut lock = self.buffer.write();
        lock[idx] = Some(packet);
    }

    /// Retrieve a cached packet by its sequence number.
    ///
    /// Returns `None` if the sequence number has not been received,
    /// or has already been evicted from the ring buffer.
    pub fn get(&self, seq: u16) -> Option<AlvioRtpPacket> {
        let idx = (seq as usize) & self.mask;
        let lock = self.buffer.read();
        if let Some(ref packet) = lock[idx] {
            if packet.header.sequence_number == seq {
                return Some(packet.clone());
            }
        }
        None
    }

    /// Retrieve multiple packets for a batch of missing sequence numbers.
    pub fn get_batch(&self, seqs: &[u16]) -> Vec<AlvioRtpPacket> {
        let lock = self.buffer.read();
        let mut res = Vec::with_capacity(seqs.len());
        for &seq in seqs {
            let idx = (seq as usize) & self.mask;
            if let Some(ref packet) = lock[idx] {
                if packet.header.sequence_number == seq {
                    res.push(packet.clone());
                }
            }
        }
        res
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for NackBuffer {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

/// Compares two 16-bit sequence numbers accounting for RFC 3550 wrap-around.
#[inline]
pub fn seq_gt(a: u16, b: u16) -> bool {
    a != b && a.wrapping_sub(b) < 0x8000
}

/// Calculates sequence distance between two 16-bit sequence numbers.
#[inline]
pub fn seq_diff(a: u16, b: u16) -> u16 {
    a.wrapping_sub(b)
}

struct MissingEntry {
    retries: u8,
    next_retry_at: Instant,
}

/// Ingress Loss Detector for identifying missing RTP sequence numbers and generating NACKs.
pub struct NackGenerator {
    max_seq: Option<u16>,
    missing: HashMap<u16, MissingEntry>,
    max_retries: u8,
    retry_interval: Duration,
    max_gap: u16,
}

impl NackGenerator {
    pub fn new(max_retries: u8, retry_interval: Duration) -> Self {
        Self {
            max_seq: None,
            missing: HashMap::new(),
            max_retries,
            retry_interval,
            max_gap: 250, // Ignore gaps larger than this (likely stream reset)
        }
    }

    /// Process an incoming packet's sequence number and register detected gaps.
    pub fn on_packet(&mut self, seq: u16, now: Instant) {
        // If this packet was previously marked missing, remove it from missing list
        self.missing.remove(&seq);

        let Some(max_seq) = self.max_seq else {
            self.max_seq = Some(seq);
            return;
        };

        if seq_gt(seq, max_seq) {
            let diff = seq.wrapping_sub(max_seq);
            if diff > 1 && diff <= self.max_gap {
                // Gap detected: record all missing sequence numbers between (max_seq + 1) and (seq - 1)
                let mut missing_seq = max_seq.wrapping_add(1);
                while missing_seq != seq {
                    self.missing.entry(missing_seq).or_insert(MissingEntry {
                        retries: 0,
                        next_retry_at: now,
                    });
                    missing_seq = missing_seq.wrapping_add(1);
                }
            }
            self.max_seq = Some(seq);
        }
    }

    /// Returns a list of sequence numbers that need NACK retransmission requests sent upstream.
    pub fn generate_nacks(&mut self, now: Instant) -> Vec<u16> {
        let mut to_request = Vec::new();
        let mut to_remove = Vec::new();

        for (&seq, entry) in self.missing.iter_mut() {
            if now >= entry.next_retry_at {
                if entry.retries < self.max_retries {
                    to_request.push(seq);
                    entry.retries += 1;
                    entry.next_retry_at = now + self.retry_interval;
                } else {
                    // Exceeded max retries: discard packet as unrecoverable
                    to_remove.push(seq);
                }
            }
        }

        for seq in to_remove {
            self.missing.remove(&seq);
        }

        to_request.sort_unstable_by(|a, b| {
            if a == b {
                std::cmp::Ordering::Equal
            } else if seq_gt(*a, *b) {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        });

        to_request
    }

    pub fn missing_count(&self) -> usize {
        self.missing.len()
    }
}

impl Default for NackGenerator {
    fn default() -> Self {
        Self::new(3, Duration::from_millis(50))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alvio_webrtc::RtpHeader;
    use bytes::Bytes;

    fn make_packet(seq: u16) -> AlvioRtpPacket {
        AlvioRtpPacket {
            header: RtpHeader {
                version: 2,
                has_padding: false,
                has_extension: false,
                csrc_count: 0,
                marker: false,
                payload_type: 96,
                sequence_number: seq,
                timestamp: 1000 + (seq as u32) * 160,
                ssrc: 0x12345678,
            },
            payload: Bytes::from_static(b"packet-payload-data"),
        }
    }

    #[test]
    fn test_nack_buffer_put_and_get() {
        let buffer = NackBuffer::new(128);

        let p1 = make_packet(10);
        let p2 = make_packet(11);
        let p3 = make_packet(12);

        buffer.put(p1);
        buffer.put(p2);
        buffer.put(p3);

        // Found in buffer
        assert!(buffer.get(10).is_some());
        assert_eq!(buffer.get(10).unwrap().header.sequence_number, 10);
        assert_eq!(buffer.get(11).unwrap().header.sequence_number, 11);
        assert_eq!(buffer.get(12).unwrap().header.sequence_number, 12);

        // Not in buffer
        assert!(buffer.get(13).is_none());
        assert!(buffer.get(9).is_none());

        // Batch get
        let batch = buffer.get_batch(&[10, 12, 99]);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].header.sequence_number, 10);
        assert_eq!(batch[1].header.sequence_number, 12);
    }

    #[test]
    fn test_nack_buffer_wrap_eviction() {
        let buffer = NackBuffer::new(64); // Capacity 64 (mask 63)

        buffer.put(make_packet(0));
        assert!(buffer.get(0).is_some());

        // Overwrite slot 0 with sequence 64
        buffer.put(make_packet(64));
        assert!(buffer.get(0).is_none()); // Evicted
        assert!(buffer.get(64).is_some());
    }

    #[test]
    fn test_nack_generator_gap_detection() {
        let mut gen = NackGenerator::new(3, Duration::from_millis(100));
        let now = Instant::now();

        gen.on_packet(100, now);
        assert_eq!(gen.missing_count(), 0);

        gen.on_packet(101, now);
        assert_eq!(gen.missing_count(), 0);

        // Gap: 102, 103 are missing, 104 arrives
        gen.on_packet(104, now);
        assert_eq!(gen.missing_count(), 2);

        let nacks = gen.generate_nacks(now);
        assert_eq!(nacks.len(), 2);
        assert!(nacks.contains(&102));
        assert!(nacks.contains(&103));

        // Immediately calling again should return empty because of retry_interval
        let nacks_immediate = gen.generate_nacks(now);
        assert!(nacks_immediate.is_empty());

        // Late arrival of 102 resolves it
        gen.on_packet(102, now);
        assert_eq!(gen.missing_count(), 1);

        // Advance time past retry interval
        let later = now + Duration::from_millis(150);
        let nacks_later = gen.generate_nacks(later);
        assert_eq!(nacks_later, vec![103]);
    }

    #[test]
    fn test_seq_gt_wraparound() {
        assert!(seq_gt(10, 5));
        assert!(!seq_gt(5, 10));
        assert!(!seq_gt(10, 10));

        // 16-bit wrap-around: 0 is greater than 65535
        assert!(seq_gt(0, 65535));
        assert!(seq_gt(5, 65530));
        assert!(!seq_gt(65535, 0));
    }
}
