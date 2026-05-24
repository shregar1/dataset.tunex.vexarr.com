use std::{
    fs,
    hint::black_box,
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use lz4_flex::frame::FrameEncoder;
use tunex_dataset::{CompressionKind, DatasetInputReader};

const LINE_COUNT: usize = 32_768;

fn input() -> Vec<u8> {
    let mut data = Vec::new();
    for index in 0..LINE_COUNT {
        writeln!(
            data,
            "{{\"id\":\"sample-{index}\",\"text\":\"hello world compressed\"}}"
        )
        .expect("write");
    }
    data
}

fn bench_compression(c: &mut Criterion) {
    let base = std::env::temp_dir().join(format!(
        "tunex-compression-bench-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let plain = base.with_extension("jsonl");
    let zstd_path = base.with_extension("jsonl.zst");
    let lz4_path = base.with_extension("jsonl.lz4");
    let data = input();
    fs::write(&plain, &data).expect("plain");
    fs::write(
        &zstd_path,
        zstd::stream::encode_all(data.as_slice(), 0).expect("zstd"),
    )
    .expect("zstd write");
    let mut encoder = FrameEncoder::new(Vec::new());
    encoder.write_all(&data).expect("lz4 input");
    fs::write(&lz4_path, encoder.finish().expect("lz4")).expect("lz4 write");

    let mut group = c.benchmark_group("compression");
    group.throughput(Throughput::Bytes(u64::try_from(data.len()).expect("bytes")));

    group.bench_function("plain_stream_throughput", |b| {
        b.iter(|| read_all(&plain, CompressionKind::None));
    });
    group.bench_function("zstd_decompression_throughput", |b| {
        b.iter(|| read_all(&zstd_path, CompressionKind::Zstd));
    });
    group.bench_function("lz4_decompression_throughput", |b| {
        b.iter(|| read_all(&lz4_path, CompressionKind::Lz4));
    });

    group.finish();
    for path in [plain, zstd_path, lz4_path] {
        fs::remove_file(path).expect("cleanup");
    }
}

fn read_all(path: &std::path::Path, compression: CompressionKind) {
    let mut reader = DatasetInputReader::open(path, compression).expect("reader");
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("read");
    black_box((buffer.len(), reader.stats()));
}

criterion_group!(benches, bench_compression);
criterion_main!(benches);
