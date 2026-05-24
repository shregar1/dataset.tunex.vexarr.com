//! Distributed epoch synchronization descriptors.

use crate::sampling::SamplingSeed;

/// Epoch state shared across distributed dataset workers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DistributedEpoch {
    /// Current epoch.
    pub epoch: u64,
    /// Base seed shared by all ranks.
    pub seed: u64,
}

impl DistributedEpoch {
    /// Creates epoch state from a base seed.
    #[must_use]
    pub const fn new(epoch: u64, seed: u64) -> Self {
        Self { epoch, seed }
    }

    /// Converts this epoch into the existing sampling seed contract.
    #[must_use]
    pub const fn sampling_seed(self) -> SamplingSeed {
        SamplingSeed::new(self.seed, self.epoch)
    }

    /// Returns the next epoch with the same base seed.
    #[must_use]
    pub const fn next(self) -> Self {
        Self {
            epoch: self.epoch + 1,
            seed: self.seed,
        }
    }
}

impl Default for DistributedEpoch {
    fn default() -> Self {
        Self { epoch: 0, seed: 42 }
    }
}
