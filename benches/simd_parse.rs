use std::hint::black_box;

use bytes::Bytes;
use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde_json::Value;
use tunex_dataset::{JsonlDatasetConfig, SimdJsonlParser};

const LINE: &[u8] = br#"{"id":"sample-1","text":"hello world from simd parse"}"#;
const LINE_COUNT: usize = 65_536;

fn bench_simd_parse(c: &mut Criterion) {
    let parser = SimdJsonlParser::new(&JsonlDatasetConfig::default());
    let lines: Vec<_> = (0..LINE_COUNT)
        .map(|_| Bytes::copy_from_slice(LINE))
        .collect();
    let bytes = u64::try_from(LINE.len() * LINE_COUNT).expect("bytes");

    let mut group = c.benchmark_group("simd_parse");
    group.throughput(Throughput::Bytes(bytes));

    group.bench_function("simd_json_fast_path", |b| {
        b.iter(|| {
            for (index, line) in lines.iter().enumerate() {
                black_box(
                    parser
                        .parse_bytes(u64::try_from(index + 1).expect("line"), line.clone())
                        .expect("parse"),
                );
            }
        });
    });

    group.bench_function("serde_json_value_baseline", |b| {
        b.iter(|| {
            for line in &lines {
                let value: Value = serde_json::from_slice(line).expect("serde");
                black_box(value["text"].as_str().expect("text"));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_simd_parse);
criterion_main!(benches);
