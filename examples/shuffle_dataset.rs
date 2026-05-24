use tunex_dataset::prelude::*;

fn main() {
    let plan = SamplingPlan::new(SamplingStrategy::RandomShuffle, SamplingSeed::new(42, 0))
        .with_max_samples(8);
    let first_epoch = plan.indices(32);

    let second_epoch = SamplingPlan::new(SamplingStrategy::RandomShuffle, SamplingSeed::new(42, 1))
        .with_max_samples(8)
        .indices(32);

    println!("epoch0={first_epoch:?}");
    println!("epoch1={second_epoch:?}");
}
