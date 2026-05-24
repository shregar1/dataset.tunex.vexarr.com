use std::{
    fs,
    hint::black_box,
    time::{SystemTime, UNIX_EPOCH},
};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{JsonlDataset, MmapJsonlDataset};

const LINE_COUNT: usize = 32_768;

fn fixture() -> (std::path::PathBuf, usize) {
    let path = std::env::temp_dir().join(format!(
        "tunex-mmap-bench-{}.jsonl",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let mut input = String::new();
    for index in 0..LINE_COUNT {
        input.push_str(&format!(
            "{{\"id\":\"sample-{index}\",\"text\":\"hello world from mmap\"}}\n"
        ));
    }
    let bytes = input.len();
    fs::write(&path, input).expect("fixture");
    (path, bytes)
}

fn bench_mmap_read(c: &mut Criterion) {
    let (path, bytes) = fixture();
    let mut group = c.benchmark_group("mmap_read");
    group.throughput(Throughput::Bytes(u64::try_from(bytes).expect("bytes")));

    group.bench_function("mmap_random_access_parse", |b| {
        b.iter(|| {
            let dataset = MmapJsonlDataset::open(&path).expect("mmap");
            for index in 0..dataset.len() {
                black_box(dataset.get(index).expect("sample"));
            }
        });
    });

    group.bench_function("buffered_non_mmap_reader_baseline", |b| {
        b.iter(|| {
            let mut dataset = JsonlDataset::open(&path).expect("jsonl");
            let mut samples = 0_usize;
            while let Some(sample) = dataset.next_sample().expect("sample") {
                samples = samples.saturating_add(1);
                black_box(sample);
            }
            black_box(samples);
        });
    });

    group.finish();
    fs::remove_file(path).expect("cleanup");
}

criterion_group!(benches, bench_mmap_read);
criterion_main!(benches);
