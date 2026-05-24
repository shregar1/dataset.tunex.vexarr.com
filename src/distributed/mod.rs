//! Distributed dataset sharding contracts.

pub mod assignment;
pub mod epoch;
pub mod partition;
pub mod rank;
pub mod shard;

pub use assignment::{DistributedBatchOwnership, ShardAssignment};
pub use epoch::DistributedEpoch;
pub use partition::{PartitionStrategy, WeightedPartition};
pub use rank::DistributedRank;
pub use shard::{DatasetShard, ShardCursor};
