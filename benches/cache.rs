use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{DatasetIdentity, DatasetManifestRegistry, ManifestBuilder};

const LOOKUPS: usize = 100_000;

fn bench_cache(c: &mut Criterion) {
    let manifest = ManifestBuilder::new(DatasetIdentity::new("train", "v1"), "jsonl")
        .with_sample_count(LOOKUPS as u64)
        .build();
    let fingerprint = manifest.fingerprint.clone();
    let mut registry = DatasetManifestRegistry::default();
    registry.register(manifest);

    let mut group = c.benchmark_group("cache");
    group.throughput(Throughput::Elements(
        u64::try_from(LOOKUPS).expect("lookups"),
    ));
    group.bench_function("manifest_cache_lookup_hit_ratio", |b| {
        b.iter(|| {
            let mut hits = 0_usize;
            for _ in 0..LOOKUPS {
                if registry.get(&fingerprint).is_some() {
                    hits = hits.saturating_add(1);
                }
            }
            black_box(hits as f64 / LOOKUPS as f64);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_cache);
criterion_main!(benches);
