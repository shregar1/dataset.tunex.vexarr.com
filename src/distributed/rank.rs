//! Distributed rank descriptors.

use crate::{DatasetError, DatasetResult};

/// Identifies one worker in a distributed dataset world.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DistributedRank {
    /// Current rank.
    pub rank: usize,
    /// Number of participating ranks.
    pub world_size: usize,
}

impl DistributedRank {
    /// Creates a validated distributed rank.
    pub fn new(rank: usize, world_size: usize) -> DatasetResult<Self> {
        let descriptor = Self { rank, world_size };
        descriptor.validate()?;
        Ok(descriptor)
    }

    /// Validates that the rank belongs to the world.
    pub fn validate(self) -> DatasetResult<()> {
        if self.world_size == 0 {
            return Err(DatasetError::InvalidConfig(
                "distributed world_size must be greater than zero".to_owned(),
            ));
        }
        if self.rank >= self.world_size {
            return Err(DatasetError::InvalidConfig(format!(
                "distributed rank {} must be less than world_size {}",
                self.rank, self.world_size
            )));
        }

        Ok(())
    }
}
