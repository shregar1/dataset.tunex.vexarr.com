//! Reservoir sampling.

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::SamplingSeed;

/// Deterministic reservoir sampler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReservoirSampler {
    /// Maximum number of retained samples.
    pub capacity: usize,
    /// Sampling seed.
    pub seed: SamplingSeed,
}

impl ReservoirSampler {
    /// Samples indices using reservoir sampling.
    #[must_use]
    pub fn sample(self, total: usize) -> Vec<usize> {
        let mut reservoir = Vec::with_capacity(self.capacity.min(total));
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed.mixed());

        for index in 0..total {
            if reservoir.len() < self.capacity {
                reservoir.push(index);
            } else {
                let slot = rng.random_range(0..=index);
                if slot < self.capacity {
                    reservoir[slot] = index;
                }
            }
        }

        reservoir
    }
}
