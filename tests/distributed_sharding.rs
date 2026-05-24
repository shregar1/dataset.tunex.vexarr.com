use std::collections::BTreeSet;

use tunex_dataset::{
    DatasetShard, DistributedBatchOwnership, DistributedEpoch, DistributedRank, PartitionStrategy,
    ShardAssignment, WeightedPartition,
};

#[test]
fn contiguous_assignment_covers_dataset_without_duplicates() {
    let assignment = ShardAssignment::build(
        32,
        8,
        DistributedEpoch::new(0, 42),
        PartitionStrategy::Contiguous,
    )
    .expect("assignment");

    assert_unique_and_complete(&assignment);
    assert_eq!(
        assignment.shard_for_rank(0).expect("rank 0").indices,
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        assignment.shard_for_rank(7).expect("rank 7").indices,
        vec![28, 29, 30, 31]
    );
}

#[test]
fn round_robin_assignment_is_rank_aware() {
    let assignment = ShardAssignment::build(
        10,
        3,
        DistributedEpoch::new(0, 42),
        PartitionStrategy::RoundRobin,
    )
    .expect("assignment");

    assert_unique_and_complete(&assignment);
    assert_eq!(
        assignment.shard_for_rank(1).expect("rank 1").indices,
        vec![1, 4, 7]
    );
    assert_eq!(assignment.owner_of(8), Some(2));
}

#[test]
fn hash_assignment_is_deterministic_for_same_seed() {
    let first = ShardAssignment::build(
        128,
        4,
        DistributedEpoch::new(3, 99),
        PartitionStrategy::Hash { seed: 77 },
    )
    .expect("first");
    let second = ShardAssignment::build(
        128,
        4,
        DistributedEpoch::new(3, 99),
        PartitionStrategy::Hash { seed: 77 },
    )
    .expect("second");

    assert_eq!(first, second);
    first.validate_no_duplicates().expect("no duplicates");
}

#[test]
fn weighted_assignment_respects_rank_weights() {
    let assignment = ShardAssignment::build(
        100,
        2,
        DistributedEpoch::default(),
        PartitionStrategy::Weighted(WeightedPartition::new(vec![0.25, 0.75])),
    )
    .expect("assignment");

    assert_unique_and_complete(&assignment);
    assert_eq!(
        assignment.shard_for_rank(0).expect("rank 0").indices.len(),
        25
    );
    assert_eq!(
        assignment.shard_for_rank(1).expect("rank 1").indices.len(),
        75
    );
}

#[test]
fn shard_cursor_is_resume_safe() {
    let shard = DatasetShard::build(
        10,
        DistributedRank::new(0, 2).expect("rank"),
        DistributedEpoch::new(0, 42),
        PartitionStrategy::RoundRobin,
    )
    .expect("shard");
    let mut cursor = shard.cursor(2);

    assert_eq!(cursor.next_index(), Some(4));
    assert_eq!(cursor.next_index(), Some(6));
    assert_eq!(cursor.resume_offset(), 4);
}

#[test]
fn batch_ownership_tracks_next_resume_offset() {
    let ownership = DistributedBatchOwnership {
        rank: DistributedRank::new(3, 8).expect("rank"),
        epoch: DistributedEpoch::new(5, 42),
        start_offset: 12,
        len: 4,
    };

    assert_eq!(ownership.next_resume_offset(), 16);
}

fn assert_unique_and_complete(assignment: &ShardAssignment) {
    let mut seen = BTreeSet::new();
    for shard in &assignment.shards {
        for index in &shard.indices {
            assert!(seen.insert(*index), "duplicate index {index}");
        }
    }

    let expected: BTreeSet<_> = (0..assignment.dataset_len).collect();
    assert_eq!(seen, expected);
}
