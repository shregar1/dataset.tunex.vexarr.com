//! Sampling strategy planning.

use super::{
    reservoir::ReservoirSampler, seed::SamplingSeed, shuffle::shuffle_indices,
    weighted::WeightedSampler,
};

/// Distributed shard descriptor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShardSpec {
    /// Rank of this worker.
    pub rank: usize,
    /// Total number of ranks.
    pub world_size: usize,
}

impl ShardSpec {
    /// Creates a shard descriptor.
    #[must_use]
    pub const fn new(rank: usize, world_size: usize) -> Self {
        Self { rank, world_size }
    }
}

/// Sampling strategy.
#[derive(Clone, Debug, PartialEq)]
pub enum SamplingStrategy {
    /// Preserve source order.
    Sequential,
    /// Shuffle all indices deterministically.
    RandomShuffle,
    /// Reservoir shuffle with bounded retained samples.
    ReservoirShuffle {
        /// Reservoir size.
        capacity: usize,
    },
    /// Weighted sampling with replacement.
    WeightedSampling {
        /// Per-index weights.
        weights: Vec<f64>,
        /// Number of output samples.
        count: usize,
    },
    /// Repeat the dataset for multiple passes.
    Repeat {
        /// Number of repeats.
        times: usize,
    },
    /// Shard indices by distributed rank.
    ShardByRank {
        /// Shard descriptor.
        shard: ShardSpec,
    },
}

/// Sampling plan with resume, range, sharding, and max sample controls.
#[derive(Clone, Debug, PartialEq)]
pub struct SamplingPlan {
    /// Strategy to apply.
    pub strategy: SamplingStrategy,
    /// Deterministic seed.
    pub seed: SamplingSeed,
    /// Optional maximum output samples.
    pub max_samples: Option<usize>,
    /// Number of initial output samples to skip.
    pub skip: usize,
    /// Optional number of samples to take after skip.
    pub take: Option<usize>,
    /// Resume offset in the planned order.
    pub resume_offset: usize,
    /// Optional distributed shard applied after strategy.
    pub shard: Option<ShardSpec>,
}

impl SamplingPlan {
    /// Creates a sampling plan.
    #[must_use]
    pub const fn new(strategy: SamplingStrategy, seed: SamplingSeed) -> Self {
        Self {
            strategy,
            seed,
            max_samples: None,
            skip: 0,
            take: None,
            resume_offset: 0,
            shard: None,
        }
    }

    /// Adds max samples.
    #[must_use]
    pub const fn with_max_samples(mut self, max_samples: usize) -> Self {
        self.max_samples = Some(max_samples);
        self
    }

    /// Adds skip/take range.
    #[must_use]
    pub const fn with_range(mut self, skip: usize, take: Option<usize>) -> Self {
        self.skip = skip;
        self.take = take;
        self
    }

    /// Adds resume offset.
    #[must_use]
    pub const fn with_resume_offset(mut self, resume_offset: usize) -> Self {
        self.resume_offset = resume_offset;
        self
    }

    /// Adds distributed sharding.
    #[must_use]
    pub const fn with_shard(mut self, shard: ShardSpec) -> Self {
        self.shard = Some(shard);
        self
    }

    /// Builds the planned index order for a dataset length.
    #[must_use]
    pub fn indices(&self, len: usize) -> Vec<usize> {
        let mut indices = match &self.strategy {
            SamplingStrategy::Sequential => (0..len).collect(),
            SamplingStrategy::RandomShuffle => shuffle_indices((0..len).collect(), self.seed),
            SamplingStrategy::ReservoirShuffle { capacity } => ReservoirSampler {
                capacity: *capacity,
                seed: self.seed,
            }
            .sample(len),
            SamplingStrategy::WeightedSampling { weights, count } => {
                WeightedSampler::new(weights.clone(), self.seed).sample(*count)
            }
            SamplingStrategy::Repeat { times } => {
                let mut repeated = Vec::with_capacity(len.saturating_mul(*times));
                for _ in 0..*times {
                    repeated.extend(0..len);
                }
                repeated
            }
            SamplingStrategy::ShardByRank { shard } => shard_indices((0..len).collect(), *shard),
        };

        if let Some(shard) = self.shard {
            indices = shard_indices(indices, shard);
        }
        let start = self
            .skip
            .saturating_add(self.resume_offset)
            .min(indices.len());
        let mut end = self.take.map_or(indices.len(), |take| {
            start.saturating_add(take).min(indices.len())
        });
        if let Some(max_samples) = self.max_samples {
            end = end.min(start.saturating_add(max_samples));
        }

        indices[start..end].to_vec()
    }
}

impl Default for SamplingPlan {
    fn default() -> Self {
        Self::new(SamplingStrategy::Sequential, SamplingSeed::default())
    }
}

fn shard_indices(indices: Vec<usize>, shard: ShardSpec) -> Vec<usize> {
    if shard.world_size == 0 {
        return Vec::new();
    }
    indices
        .into_iter()
        .enumerate()
        .filter_map(|(position, index)| {
            (position % shard.world_size == shard.rank).then_some(index)
        })
        .collect()
}
