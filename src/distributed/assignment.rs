//! Distributed shard assignment and batch ownership.

use crate::{DatasetError, DatasetResult};

use super::{DatasetShard, DistributedEpoch, DistributedRank, PartitionStrategy};

/// Full per-rank shard assignment for an epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct ShardAssignment {
    /// Synchronized epoch used for assignment.
    pub epoch: DistributedEpoch,
    /// Total dataset length.
    pub dataset_len: usize,
    /// Strategy used to partition samples.
    pub strategy: PartitionStrategy,
    /// One shard per rank.
    pub shards: Vec<DatasetShard>,
}

impl ShardAssignment {
    /// Builds a complete world assignment.
    pub fn build(
        dataset_len: usize,
        world_size: usize,
        epoch: DistributedEpoch,
        strategy: PartitionStrategy,
    ) -> DatasetResult<Self> {
        if world_size == 0 {
            return Err(DatasetError::InvalidConfig(
                "distributed world_size must be greater than zero".to_owned(),
            ));
        }

        let mut shards = Vec::with_capacity(world_size);
        for rank in 0..world_size {
            shards.push(DatasetShard::build(
                dataset_len,
                DistributedRank::new(rank, world_size)?,
                epoch,
                strategy.clone(),
            )?);
        }

        let assignment = Self {
            epoch,
            dataset_len,
            strategy,
            shards,
        };
        assignment.validate_no_duplicates()?;
        Ok(assignment)
    }

    /// Returns a rank shard.
    #[must_use]
    pub fn shard_for_rank(&self, rank: usize) -> Option<&DatasetShard> {
        self.shards.get(rank)
    }

    /// Returns the rank that owns a dataset index.
    #[must_use]
    pub fn owner_of(&self, index: usize) -> Option<usize> {
        self.shards
            .iter()
            .find(|shard| shard.indices.contains(&index))
            .map(|shard| shard.rank.rank)
    }

    /// Validates that each dataset index appears at most once.
    pub fn validate_no_duplicates(&self) -> DatasetResult<()> {
        let mut seen = vec![false; self.dataset_len];
        for shard in &self.shards {
            for index in &shard.indices {
                if *index >= self.dataset_len {
                    return Err(DatasetError::InvalidConfig(format!(
                        "shard index {index} exceeds dataset length {}",
                        self.dataset_len
                    )));
                }
                if seen[*index] {
                    return Err(DatasetError::InvalidConfig(format!(
                        "duplicate distributed dataset index {index}"
                    )));
                }
                seen[*index] = true;
            }
        }
        Ok(())
    }
}

/// Ownership metadata for a batch emitted by a distributed dataset worker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DistributedBatchOwnership {
    /// Rank that produced the batch.
    pub rank: DistributedRank,
    /// Epoch for the batch.
    pub epoch: DistributedEpoch,
    /// First shard offset included in the batch.
    pub start_offset: usize,
    /// Number of samples in the batch.
    pub len: usize,
}

impl DistributedBatchOwnership {
    /// Returns the next resume offset after this batch is fully consumed.
    #[must_use]
    pub const fn next_resume_offset(&self) -> usize {
        self.start_offset + self.len
    }
}
