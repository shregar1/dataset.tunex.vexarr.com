use tunex_core::checkpoint::{CheckpointId, CheckpointMetadata};
use tunex_dataset::{
    CheckpointRestorePlan, DatasetCursor, DatasetOffset, DatasetResumeSnapshot, DatasetResumeState,
    DistributedBatchOwnership, DistributedEpoch, DistributedRank, PartitionStrategy, RestorePolicy,
};

#[test]
fn cursor_tracks_exact_sample_line_and_byte_offset() {
    let cursor = DatasetCursor::new(3, 99)
        .with_offset(DatasetOffset::new(Some(42), Some(8192)))
        .advanced_by(128);

    assert_eq!(cursor.epoch, 3);
    assert_eq!(cursor.sample_index, 128);
    assert_eq!(cursor.line_number, Some(42));
    assert_eq!(cursor.byte_offset, Some(8192));
    assert_eq!(
        cursor.sampling_seed().mixed(),
        DistributedEpoch::new(3, 99).sampling_seed().mixed()
    );
}

#[test]
fn resume_state_preserves_distributed_shard_identity() {
    let rank = DistributedRank::new(2, 8).expect("rank");
    let state = DatasetResumeState::new(DatasetCursor::new(4, 42))
        .with_dataset_len(1_000)
        .with_distributed(rank, PartitionStrategy::RoundRobin)
        .expect("distributed");

    state.validate().expect("valid");
    assert_eq!(state.cursor.rank, Some(2));
    assert_eq!(state.cursor.shard_id, Some(2));
    assert_eq!(state.world_size, Some(8));
}

#[test]
fn cursor_advances_from_distributed_batch_ownership() {
    let mut cursor = DatasetCursor::new(0, 42)
        .with_rank(DistributedRank::new(1, 4).expect("rank"))
        .expect("cursor");
    let ownership = DistributedBatchOwnership {
        rank: DistributedRank::new(1, 4).expect("rank"),
        epoch: DistributedEpoch::new(2, 42),
        start_offset: 64,
        len: 16,
    };

    cursor.advance_batch(&ownership).expect("advance");

    assert_eq!(cursor.epoch, 2);
    assert_eq!(cursor.sample_index, 80);
}

#[test]
fn snapshot_attaches_checkpoint_identity() {
    let snapshot = DatasetResumeSnapshot::from_cursor(DatasetCursor::new(1, 7))
        .expect("snapshot")
        .with_checkpoint("ckpt-1", 500);

    assert_eq!(snapshot.version, DatasetResumeSnapshot::VERSION);
    assert_eq!(snapshot.checkpoint_id.as_deref(), Some("ckpt-1"));
    assert_eq!(snapshot.checkpoint_step, Some(500));
}

#[test]
fn strict_restore_rejects_checkpoint_step_mismatch() {
    let snapshot = DatasetResumeSnapshot::from_cursor(DatasetCursor::new(1, 7))
        .expect("snapshot")
        .with_checkpoint("ckpt-1", 500);
    let checkpoint = CheckpointMetadata::new(CheckpointId::new("ckpt-1"), 501, "mock");

    let error =
        CheckpointRestorePlan::from_snapshot(snapshot, Some(&checkpoint), RestorePolicy::Strict)
            .expect_err("mismatch");

    assert!(error.to_string().contains("does not match"));
}

#[test]
fn best_effort_restore_allows_missing_checkpoint_metadata() {
    let snapshot = DatasetResumeSnapshot::from_cursor(DatasetCursor::new(1, 7)).expect("snapshot");
    let plan = CheckpointRestorePlan::from_snapshot(snapshot, None, RestorePolicy::BestEffort)
        .expect("plan");

    assert_eq!(plan.policy, RestorePolicy::BestEffort);
    assert_eq!(plan.checkpoint_step, None);
}
