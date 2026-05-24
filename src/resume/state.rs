//! Dataset resume state validation.

use crate::{
    distributed::{DistributedRank, PartitionStrategy},
    DatasetError, DatasetResult,
};

use super::DatasetCursor;

/// Complete resume state required to reproduce dataset order.
#[derive(Clone, Debug, PartialEq)]
pub struct DatasetResumeState {
    /// Exact cursor at checkpoint time.
    pub cursor: DatasetCursor,
    /// Dataset length used when planning the order.
    pub dataset_len: Option<u64>,
    /// Distributed world size, when sharded.
    pub world_size: Option<u32>,
    /// Partition strategy used for distributed assignment.
    pub partition_strategy: Option<PartitionStrategy>,
}

impl DatasetResumeState {
    /// Creates resume state from a cursor.
    #[must_use]
    pub const fn new(cursor: DatasetCursor) -> Self {
        Self {
            cursor,
            dataset_len: None,
            world_size: None,
            partition_strategy: None,
        }
    }

    /// Adds dataset length.
    #[must_use]
    pub const fn with_dataset_len(mut self, dataset_len: u64) -> Self {
        self.dataset_len = Some(dataset_len);
        self
    }

    /// Adds distributed rank and strategy metadata.
    pub fn with_distributed(
        mut self,
        rank: DistributedRank,
        strategy: PartitionStrategy,
    ) -> DatasetResult<Self> {
        rank.validate()?;
        self.cursor.rank = Some(rank.rank as u32);
        self.cursor.shard_id = Some(rank.rank as u32);
        self.world_size = Some(rank.world_size as u32);
        self.partition_strategy = Some(strategy);
        Ok(self)
    }

    /// Validates that resume state is internally consistent.
    pub fn validate(&self) -> DatasetResult<()> {
        if let Some(world_size) = self.world_size {
            if world_size == 0 {
                return Err(DatasetError::InvalidConfig(
                    "resume world_size must be greater than zero".to_owned(),
                ));
            }
            let rank = self.cursor.rank.ok_or_else(|| {
                DatasetError::InvalidConfig("resume state with world_size requires rank".to_owned())
            })?;
            if rank >= world_size {
                return Err(DatasetError::InvalidConfig(format!(
                    "resume rank {rank} must be less than world_size {world_size}"
                )));
            }
        }

        if let Some(dataset_len) = self.dataset_len {
            if self.cursor.sample_index > dataset_len {
                return Err(DatasetError::InvalidConfig(format!(
                    "resume sample index {} exceeds dataset length {dataset_len}",
                    self.cursor.sample_index
                )));
            }
        }

        Ok(())
    }
}
