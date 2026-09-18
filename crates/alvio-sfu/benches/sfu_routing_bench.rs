use std::sync::Arc;
use alvio_core::{PeerId, StreamKind, StreamLayer, TrackId};
use alvio_sfu::data::DataMessage;
use alvio_sfu::{DataRouter, NackBuffer, RtpRouter, StreamConsumer, StreamSource};
use alvio_webrtc::{AlvioRtpPacket, RtpHeader};
use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn make_sample_packet(ssrc: u32, seq: u16, timestamp: u32) -> AlvioRtpPacket {
    let header = RtpHeader {
        version: 2,
        has_padding: false,
        has_extension: false,
        csrc_count: 0,
        marker: false,
        payload_type: 96,
        sequence_number: seq,
        timestamp,
        ssrc,
    };
    let payload = Bytes::from_static(&[0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff, 0x11, 0x22]);
    AlvioRtpPacket { header, payload }
}

fn bench_rtp_header_parse_and_rewrite(c: &mut Criterion) {
    let packet = make_sample_packet(1001, 100, 90000);
    let serialized = packet.serialize();

    c.bench_function("rtp_header_parse_and_rewrite", |b| {
        b.iter(|| {
            let parsed = AlvioRtpPacket::parse(black_box(serialized.clone())).unwrap();
            let mut modified = parsed;
            modified.header.ssrc = black_box(5005);
            modified.header.sequence_number = black_box(200);
            let out = modified.serialize();
            black_box(out);
        });
    });
}

fn bench_nack_buffer_put_and_get(c: &mut Criterion) {
    let buffer = NackBuffer::default();
    let packet = make_sample_packet(1001, 500, 90000);

    c.bench_function("nack_buffer_put_and_get", |b| {
        let mut seq = 0u16;
        b.iter(|| {
            let mut p = packet.clone();
            p.header.sequence_number = seq;
            buffer.put(p);
            let retrieved = buffer.get(black_box(seq));
            black_box(retrieved);
            seq = seq.wrapping_add(1);
        });
    });
}

fn bench_sfu_routing_fanout(c: &mut Criterion) {
    let mut group = c.benchmark_group("sfu_routing_fanout");

    for consumer_count in [10, 50, 100].iter() {
        let router = RtpRouter::new();
        let source_ssrc = 1111;
        let source = Arc::new(StreamSource::new(
            TrackId::new("trk_bench_source"),
            PeerId::new("peer_publisher"),
            source_ssrc,
            StreamKind::Video,
            90000,
        ));
        router.register_source(source);

        for i in 0..*consumer_count {
            let consumer = Arc::new(StreamConsumer::new(
                format!("consumer_{}", i),
                PeerId::new(format!("peer_sub_{}", i)),
                2000 + i as u32,
                100,
                StreamLayer::High,
            ));
            router.add_consumer(source_ssrc, consumer);
        }

        let packet = make_sample_packet(source_ssrc, 42, 90000);

        group.bench_function(format!("fanout_{}_consumers", consumer_count), |b| {
            b.iter(|| {
                let forwarded = router.route_packet(black_box(&packet));
                black_box(forwarded);
            });
        });
    }

    group.finish();
}

fn bench_data_router_broadcast(c: &mut Criterion) {
    let data_router = DataRouter::new();

    for i in 0..100 {
        let pid = PeerId::new(format!("peer_{}", i));
        data_router.register_peer(pid.clone());
        data_router.subscribe(&pid, "chat");
    }

    let msg = DataMessage {
        sender_peer_id: PeerId::new("peer_0"),
        label: "chat".to_string(),
        payload: Bytes::from_static(b"benchmark payload"),
        binary: false,
    };

    c.bench_function("data_router_broadcast_100_peers", |b| {
        b.iter(|| {
            let targets = data_router.route_broadcast(black_box(&msg));
            black_box(targets);
        });
    });
}

criterion_group!(
    benches,
    bench_rtp_header_parse_and_rewrite,
    bench_nack_buffer_put_and_get,
    bench_sfu_routing_fanout,
    bench_data_router_broadcast
);
criterion_main!(benches);
