use std::{
    fs,
    hint::black_box,
    time::{SystemTime, UNIX_EPOCH},
};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{
    BatchBuilder, DatasetFilterConfig, JsonlDataset, ManifestBuilder, StatisticsBuilder,
};

const LINE_COUNT: usize = 32_768;

fn bench_end_to_end(c: &mut Criterion) {
    let path = std::env::temp_dir().join(format!(
        "tunex-e2e-bench-{}.jsonl",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    let mut input = String::new();
    for index in 0..LINE_COUNT {
        input.push_str(&format!(
            "{{\"id\":\"sample-{index}\",\"text\":\"high quality training sample {index}\"}}\n"
        ));
    }
    fs::write(&path, &input).expect("fixture");

    let mut group = c.benchmark_group("end_to_end");
    group.throughput(Throughput::Bytes(
        u64::try_from(input.len()).expect("bytes"),
    ));
    group.bench_function("jsonl_filter_batch_manifest", |b| {
        b.iter(|| {
            let mut dataset = JsonlDataset::open(&path).expect("dataset");
            let mut filter = DatasetFilterConfig {
                min_chars: Some(10),
                remove_empty: true,
                dedupe: true,
                ..DatasetFilterConfig::default()
            }
            .build_engine()
            .expect("filter");
            let mut stats = StatisticsBuilder::default();
            let mut builder = BatchBuilder::with_capacity(1024);
            let mut batches = 0_usize;

            while let Some(sample) = dataset.next_sample().expect("sample") {
                if let Some(sample) = filter.apply(sample).expect("filter") {
                    stats.record_sample(&sample);
                    builder.push(sample);
                    if builder.len() == 1024 {
                        batches = batches.saturating_add(1);
                        black_box(builder.finish());
                    }
                }
            }

            let manifest = ManifestBuilder::from_jsonl_config(
                "bench",
                "v1",
                &tunex_dataset::JsonlDatasetConfig::new(&path),
            )
            .expect("manifest")
            .with_sample_count(LINE_COUNT as u64)
            .with_statistics(stats.finish())
            .build();
            black_box((batches, builder.finish(), filter.stats().clone(), manifest));
        });
    });
    group.finish();

    fs::remove_file(path).expect("cleanup");
}

criterion_group!(benches, bench_end_to_end);
criterion_main!(benches);
