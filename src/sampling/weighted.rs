//! Weighted sampling.

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::SamplingSeed;

/// Deterministic weighted sampler.
#[derive(Clone, Debug, PartialEq)]
pub struct WeightedSampler {
    weights: Vec<f64>,
    seed: SamplingSeed,
}

impl WeightedSampler {
    /// Creates a weighted sampler.
    #[must_use]
    pub fn new(weights: Vec<f64>, seed: SamplingSeed) -> Self {
        Self { weights, seed }
    }

    /// Samples `count` indices with replacement.
    #[must_use]
    pub fn sample(&self, count: usize) -> Vec<usize> {
        let total = self
            .weights
            .iter()
            .copied()
            .filter(|weight| *weight > 0.0)
            .sum::<f64>();
        if total <= f64::EPSILON {
            return Vec::new();
        }

        let mut rng = ChaCha8Rng::seed_from_u64(self.seed.mixed());
        let mut output = Vec::with_capacity(count);
        for _ in 0..count {
            let mut target = rng.random_range(0.0..total);
            for (index, weight) in self.weights.iter().copied().enumerate() {
                if weight <= 0.0 {
                    continue;
                }
                if target < weight {
                    output.push(index);
                    break;
                }
                target -= weight;
            }
        }
        output
    }
}
