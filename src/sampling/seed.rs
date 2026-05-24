//! Deterministic seed handling.

/// Seed descriptor for deterministic sampling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SamplingSeed {
    /// Base seed.
    pub seed: u64,
    /// Epoch number mixed into the seed.
    pub epoch: u64,
}

impl SamplingSeed {
    /// Creates a seed descriptor.
    #[must_use]
    pub const fn new(seed: u64, epoch: u64) -> Self {
        Self { seed, epoch }
    }

    /// Returns the epoch-mixed seed.
    #[must_use]
    pub const fn mixed(self) -> u64 {
        self.seed
            ^ self
                .epoch
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .rotate_left(17)
    }
}

impl Default for SamplingSeed {
    fn default() -> Self {
        Self { seed: 42, epoch: 0 }
    }
}
