//! Dataset shard descriptors and resume cursors.

use super::{DistributedEpoch, DistributedRank, PartitionStrategy};
use crate::DatasetResult;

/// Dataset indices owned by one distributed rank for one epoch.
#[derive(Clone, Debug, PartialEq)]
pub struct DatasetShard {
    /// Owning rank.
    pub rank: DistributedRank,
    /// Synchronized epoch.
    pub epoch: DistributedEpoch,
    /// Partitioning strategy used to create this shard.
    pub strategy: PartitionStrategy,
    /// Dataset indices owned by this rank.
    pub indices: Vec<usize>,
}

impl DatasetShard {
    /// Builds a shard for the given rank and dataset length.
    pub fn build(
        len: usize,
        rank: DistributedRank,
        epoch: DistributedEpoch,
        strategy: PartitionStrategy,
    ) -> DatasetResult<Self> {
        let indices = strategy.assign_indices(len, rank)?;

        Ok(Self {
            rank,
            epoch,
            strategy,
            indices,
        })
    }

    /// Creates a cursor starting at a resume offset.
    #[must_use]
    pub fn cursor(&self, resume_offset: usize) -> ShardCursor<'_> {
        ShardCursor {
            shard: self,
            offset: resume_offset.min(self.indices.len()),
        }
    }

    /// Returns true if this shard owns a dataset index.
    #[must_use]
    pub fn owns(&self, index: usize) -> bool {
        self.indices.binary_search(&index).is_ok()
    }
}

/// Resume-safe cursor over a shard assignment.
#[derive(Clone, Debug)]
pub struct ShardCursor<'a> {
    shard: &'a DatasetShard,
    offset: usize,
}

impl<'a> ShardCursor<'a> {
    /// Returns the next owned dataset index.
    pub fn next_index(&mut self) -> Option<usize> {
        let index = self.shard.indices.get(self.offset).copied()?;
        self.offset += 1;
        Some(index)
    }

    /// Returns the current resume offset.
    #[must_use]
    pub const fn resume_offset(&self) -> usize {
        self.offset
    }
}
