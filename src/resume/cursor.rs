//! Dataset cursor tracking.

use super::DatasetOffset;
use crate::{
    distributed::{DistributedBatchOwnership, DistributedEpoch, DistributedRank},
    sampling::SamplingSeed,
    DatasetError, DatasetResult,
};

/// Exact logical and physical dataset position for checkpoint resume.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetCursor {
    /// Current epoch.
    pub epoch: u64,
    /// Logical sample index in the current planned order.
    pub sample_index: u64,
    /// Source line number for line-oriented formats.
    pub line_number: Option<u64>,
    /// Source byte offset for seekable formats.
    pub byte_offset: Option<u64>,
    /// Distributed shard id.
    pub shard_id: Option<u32>,
    /// Distributed rank.
    pub rank: Option<u32>,
    /// Shuffle/sampling seed.
    pub seed: u64,
}

impl DatasetCursor {
    /// Creates a cursor at epoch start.
    #[must_use]
    pub const fn new(epoch: u64, seed: u64) -> Self {
        Self {
            epoch,
            sample_index: 0,
            line_number: None,
            byte_offset: None,
            shard_id: None,
            rank: None,
            seed,
        }
    }

    /// Creates a cursor from a sampling seed.
    #[must_use]
    pub const fn from_sampling_seed(seed: SamplingSeed) -> Self {
        Self::new(seed.epoch, seed.seed)
    }

    /// Adds source offset information.
    #[must_use]
    pub const fn with_offset(mut self, offset: DatasetOffset) -> Self {
        self.line_number = offset.line_number;
        self.byte_offset = offset.byte_offset;
        self
    }

    /// Adds distributed shard identity.
    pub fn with_rank(mut self, rank: DistributedRank) -> DatasetResult<Self> {
        rank.validate()?;
        self.rank = Some(rank.rank as u32);
        self.shard_id = Some(rank.rank as u32);
        Ok(self)
    }

    /// Advances logical sample position.
    #[must_use]
    pub const fn advanced_by(mut self, samples: u64) -> Self {
        self.sample_index += samples;
        self
    }

    /// Advances by consumed batch ownership metadata.
    pub fn advance_batch(&mut self, ownership: &DistributedBatchOwnership) -> DatasetResult<()> {
        let rank = self.rank.ok_or_else(|| {
            DatasetError::InvalidConfig("cursor rank is required for distributed resume".to_owned())
        })?;
        if rank as usize != ownership.rank.rank {
            return Err(DatasetError::InvalidConfig(format!(
                "cursor rank {rank} does not match batch rank {}",
                ownership.rank.rank
            )));
        }

        self.epoch = ownership.epoch.epoch;
        self.sample_index = ownership.next_resume_offset() as u64;
        Ok(())
    }

    /// Converts this cursor into a distributed epoch descriptor.
    #[must_use]
    pub const fn distributed_epoch(&self) -> DistributedEpoch {
        DistributedEpoch::new(self.epoch, self.seed)
    }

    /// Converts this cursor into a sampling seed descriptor.
    #[must_use]
    pub const fn sampling_seed(&self) -> SamplingSeed {
        SamplingSeed::new(self.seed, self.epoch)
    }
}

impl Default for DatasetCursor {
    fn default() -> Self {
        Self::new(0, 42)
    }
}
