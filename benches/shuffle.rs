use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{SamplingPlan, SamplingSeed, SamplingStrategy, ShardSpec};

const SAMPLE_COUNT: usize = 1_000_000;

fn bench_shuffle(c: &mut Criterion) {
    let mut group = c.benchmark_group("shuffle");
    group.throughput(Throughput::Elements(
        u64::try_from(SAMPLE_COUNT).expect("samples"),
    ));

    group.bench_function("deterministic_random_shuffle", |b| {
        let plan = SamplingPlan::new(SamplingStrategy::RandomShuffle, SamplingSeed::new(42, 7));
        b.iter(|| black_box(plan.indices(SAMPLE_COUNT)));
    });

    group.bench_function("rank_aware_shard_by_rank", |b| {
        let plan = SamplingPlan::new(
            SamplingStrategy::ShardByRank {
                shard: ShardSpec::new(3, 8),
            },
            SamplingSeed::new(42, 7),
        );
        b.iter(|| black_box(plan.indices(SAMPLE_COUNT)));
    });

    group.finish();
}

criterion_group!(benches, bench_shuffle);
criterion_main!(benches);
