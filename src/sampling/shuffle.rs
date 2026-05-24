//! Deterministic shuffle helpers.

use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::SamplingSeed;

/// Shuffles indices deterministically.
#[must_use]
pub fn shuffle_indices(mut indices: Vec<usize>, seed: SamplingSeed) -> Vec<usize> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed.mixed());
    for index in (1..indices.len()).rev() {
        let swap = rng.random_range(0..=index);
        indices.swap(index, swap);
    }
    indices
}
