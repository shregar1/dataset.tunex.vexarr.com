use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use tunex_dataset::{DistributedEpoch, PartitionStrategy, ShardAssignment, WeightedPartition};

const SAMPLE_COUNT: usize = 1_000_000;

fn bench_distributed(c: &mut Criterion) {
    let mut group = c.benchmark_group("distributed");
    group.throughput(Throughput::Elements(
        u64::try_from(SAMPLE_COUNT).expect("samples"),
    ));

    group.bench_function("contiguous_8_rank_assignment", |b| {
        b.iter(|| {
            black_box(
                ShardAssignment::build(
                    SAMPLE_COUNT,
                    8,
                    DistributedEpoch::new(0, 42),
                    PartitionStrategy::Contiguous,
                )
                .expect("assignment"),
            );
        });
    });

    group.bench_function("hash_8_rank_assignment", |b| {
        b.iter(|| {
            black_box(
                ShardAssignment::build(
                    SAMPLE_COUNT,
                    8,
                    DistributedEpoch::new(0, 42),
                    PartitionStrategy::Hash { seed: 42 },
                )
                .expect("assignment"),
            );
        });
    });

    group.bench_function("weighted_8_rank_assignment", |b| {
        b.iter(|| {
            black_box(
                ShardAssignment::build(
                    SAMPLE_COUNT,
                    8,
                    DistributedEpoch::new(0, 42),
                    PartitionStrategy::Weighted(WeightedPartition::new(vec![1.0; 8])),
                )
                .expect("assignment"),
            );
        });
    });

    group.finish();
}

criterion_group!(benches, bench_distributed);
criterion_main!(benches);
