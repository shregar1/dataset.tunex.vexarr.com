//! Dataset manifest and indexing layer.

pub mod fingerprint;
pub mod manifest;
pub mod metadata;
pub mod registry;
pub mod statistics;

pub use fingerprint::{DatasetFingerprint, FingerprintBuilder};
pub use manifest::{DatasetManifest, ManifestBuilder};
pub use metadata::{
    CacheMetadata, CompressionMetadata, DatasetIdentity, SchemaMetadata, ShardMetadata,
    SplitMetadata,
};
pub use registry::DatasetManifestRegistry;
pub use statistics::{DatasetStatistics, StatisticsBuilder};
