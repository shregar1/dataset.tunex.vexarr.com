use tunex_dataset::{SamplingPlan, SamplingSeed, SamplingStrategy, ShardSpec};

#[test]
fn same_seed_and_epoch_produce_same_shuffle() {
    let plan = SamplingPlan::new(SamplingStrategy::RandomShuffle, SamplingSeed::new(123, 0));

    assert_eq!(plan.indices(16), plan.indices(16));
}

#[test]
fn different_epoch_changes_shuffle_order() {
    let first =
        SamplingPlan::new(SamplingStrategy::RandomShuffle, SamplingSeed::new(123, 0)).indices(16);
    let second =
        SamplingPlan::new(SamplingStrategy::RandomShuffle, SamplingSeed::new(123, 1)).indices(16);

    assert_ne!(first, second);
}

#[test]
fn resume_offset_skip_take_and_max_samples_are_applied() {
    let plan = SamplingPlan::new(SamplingStrategy::Sequential, SamplingSeed::default())
        .with_range(2, Some(5))
        .with_resume_offset(1)
        .with_max_samples(3);

    assert_eq!(plan.indices(10), vec![3, 4, 5]);
}

#[test]
fn shard_by_rank_splits_deterministic_order() {
    let plan = SamplingPlan::new(
        SamplingStrategy::ShardByRank {
            shard: ShardSpec::new(1, 3),
        },
        SamplingSeed::default(),
    );

    assert_eq!(plan.indices(10), vec![1, 4, 7]);
}

#[test]
fn repeat_strategy_repeats_dataset_indices() {
    let plan = SamplingPlan::new(
        SamplingStrategy::Repeat { times: 3 },
        SamplingSeed::default(),
    );

    assert_eq!(plan.indices(3), vec![0, 1, 2, 0, 1, 2, 0, 1, 2]);
}

#[test]
fn reservoir_and_weighted_sampling_are_deterministic() {
    let reservoir = SamplingPlan::new(
        SamplingStrategy::ReservoirShuffle { capacity: 4 },
        SamplingSeed::new(77, 2),
    );
    let weighted = SamplingPlan::new(
        SamplingStrategy::WeightedSampling {
            weights: vec![0.0, 10.0, 0.0],
            count: 4,
        },
        SamplingSeed::new(77, 2),
    );

    assert_eq!(reservoir.indices(32), reservoir.indices(32));
    assert_eq!(weighted.indices(3), vec![1, 1, 1, 1]);
}
