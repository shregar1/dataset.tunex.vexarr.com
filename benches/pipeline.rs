use std::{
    fs,
    hint::black_box,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{JsonlDatasetConfig, ParallelJsonlPipeline, PipelineOptions};

const LINE_COUNT: usize = 65_536;

fn bench_parallel_pipeline(c: &mut Criterion) {
    let path = std::env::temp_dir().join(format!(
        "tunex-pipeline-bench-{}.jsonl",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos()
    ));
    let mut input = String::new();
    for index in 0..LINE_COUNT {
        input.push_str(&format!(
            "{{\"id\":\"sample-{index}\",\"text\":\"hello world from tunex dataset\"}}\n"
        ));
    }
    fs::write(&path, &input).expect("benchmark fixture should write");

    let mut group = c.benchmark_group("parallel_pipeline");
    group.throughput(Throughput::Bytes(
        u64::try_from(input.len()).expect("fixture length should fit u64"),
    ));
    group.bench_function("samples_sec_mb_sec_queue_stall", |b| {
        b.iter(|| {
            let mut pipeline = ParallelJsonlPipeline::open(
                JsonlDatasetConfig::new(&path),
                PipelineOptions {
                    chunk_lines: 4096,
                    chunk_queue_capacity: 4,
                    batch_queue_capacity: 4,
                    send_timeout: Duration::from_secs(1),
                },
            )
            .expect("pipeline should open");

            let mut samples = 0_usize;
            while let Some(batch) = pipeline.next_batch().expect("batch should read") {
                samples = samples.saturating_add(batch.len());
                black_box(batch);
            }
            let stats = pipeline.join().expect("pipeline should join");
            black_box((samples, stats.queue_stall_time));
        });
    });
    group.finish();

    fs::remove_file(path).expect("benchmark fixture should remove");
}

criterion_group!(benches, bench_parallel_pipeline);
criterion_main!(benches);
