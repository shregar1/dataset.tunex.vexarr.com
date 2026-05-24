//! Deterministic distributed partition strategies.

use crate::{DatasetError, DatasetResult};

use super::DistributedRank;

/// Strategy used to partition dataset indices across ranks.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum PartitionStrategy {
    /// Assign contiguous index ranges to ranks.
    Contiguous,
    /// Assign every Nth sample to rank N.
    #[default]
    RoundRobin,
    /// Assign by deterministic hash of sample index and seed.
    Hash { seed: u64 },
    /// Assign contiguous weighted ranges by rank weight.
    Weighted(WeightedPartition),
}

impl PartitionStrategy {
    /// Returns indices assigned to one rank.
    pub fn assign_indices(&self, len: usize, rank: DistributedRank) -> DatasetResult<Vec<usize>> {
        rank.validate()?;
        match self {
            Self::Contiguous => Ok(contiguous_indices(len, rank)),
            Self::RoundRobin => Ok(round_robin_indices(len, rank)),
            Self::Hash { seed } => Ok(hash_indices(len, rank, *seed)),
            Self::Weighted(partition) => partition.assign_indices(len, rank),
        }
    }
}

/// Rank weights used by weighted partitioning.
#[derive(Clone, Debug, PartialEq)]
pub struct WeightedPartition {
    /// Per-rank weights.
    pub weights: Vec<f64>,
}

impl WeightedPartition {
    /// Creates weighted partitioning.
    #[must_use]
    pub fn new(weights: Vec<f64>) -> Self {
        Self { weights }
    }

    /// Returns indices for this rank.
    pub fn assign_indices(&self, len: usize, rank: DistributedRank) -> DatasetResult<Vec<usize>> {
        rank.validate()?;
        if self.weights.len() != rank.world_size {
            return Err(DatasetError::InvalidConfig(format!(
                "weighted partition requires {} weights, got {}",
                rank.world_size,
                self.weights.len()
            )));
        }
        if self.weights.iter().any(|weight| *weight < 0.0) {
            return Err(DatasetError::InvalidConfig(
                "weighted partition weights must be non-negative".to_owned(),
            ));
        }
        let total: f64 = self.weights.iter().sum();
        if total <= 0.0 {
            return Err(DatasetError::InvalidConfig(
                "weighted partition total weight must be greater than zero".to_owned(),
            ));
        }

        let mut start = 0usize;
        for current_rank in 0..rank.world_size {
            let end = if current_rank + 1 == rank.world_size {
                len
            } else {
                weighted_boundary(len, &self.weights, total, current_rank + 1)
            };

            if current_rank == rank.rank {
                return Ok((start..end).collect());
            }
            start = end;
        }

        Ok(Vec::new())
    }
}

fn contiguous_indices(len: usize, rank: DistributedRank) -> Vec<usize> {
    let start = len.saturating_mul(rank.rank) / rank.world_size;
    let end = len.saturating_mul(rank.rank + 1) / rank.world_size;
    (start..end).collect()
}

fn round_robin_indices(len: usize, rank: DistributedRank) -> Vec<usize> {
    (rank.rank..len).step_by(rank.world_size).collect()
}

fn hash_indices(len: usize, rank: DistributedRank, seed: u64) -> Vec<usize> {
    (0..len)
        .filter(|index| {
            stable_partition_hash(*index as u64, seed) as usize % rank.world_size == rank.rank
        })
        .collect()
}

fn weighted_boundary(len: usize, weights: &[f64], total: f64, rank_count: usize) -> usize {
    let prefix: f64 = weights.iter().take(rank_count).sum();
    ((prefix / total) * len as f64).round() as usize
}

fn stable_partition_hash(index: u64, seed: u64) -> u64 {
    let mut value = index ^ seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}
