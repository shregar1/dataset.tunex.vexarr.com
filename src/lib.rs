//! Dataset loading engine for the `TuneX` ecosystem.
//!
//! The v0.1 surface focuses on JSONL records that can be read through bounded
//! streaming or memory-mapped readers and emitted as `tunex-core` `RawSample`
//! values through the `DatasetSource` contract.

pub mod batch;
pub mod compression;
pub mod config;
pub mod distributed;
pub mod error;
pub mod filter;
pub mod index;
pub mod jsonl;
pub mod metrics;
pub mod mmap;
pub mod parquet;
pub mod pipeline;
pub mod plugin;
pub mod prelude;
pub mod reader;
pub mod resume;
pub mod sampling;
pub mod source;

pub use batch::{
    BatchBuilder, BatchMetadata, BatchSampler, BatchStats, DatasetBatch, PrefetchBatches,
};
pub use compression::{CompressionFormat, CompressionStats, DatasetInputReader};
pub use config::{
    CompressionKind, ConfigValidationError, ConfigValidationReport, DatasetFormat, DatasetMode,
    DatasetPipelineConfig, JsonlDatasetConfig, ValidateDatasetConfig,
};
pub use distributed::{
    DatasetShard, DistributedBatchOwnership, DistributedEpoch, DistributedRank, PartitionStrategy,
    ShardAssignment, ShardCursor, WeightedPartition,
};
pub use error::{DatasetError, DatasetResult};
pub use filter::{
    DatasetFilter, DatasetFilterConfig, DedupeFilter, FilterDecision, FilterEngine, FilterReason,
    FilterStats, LanguageFilter, LengthFilter, PiiFilter, RegexFilter,
};
pub use index::{
    CacheMetadata, CompressionMetadata, DatasetFingerprint, DatasetIdentity, DatasetManifest,
    DatasetManifestRegistry, DatasetStatistics, FingerprintBuilder, ManifestBuilder,
    SchemaMetadata, ShardMetadata, SplitMetadata, StatisticsBuilder,
};
pub use jsonl::{
    DecodedJsonlRecord, DecodedText, FastPathJsonlParser, JsonlDataset, JsonlDatasetSource,
    JsonlParser, JsonlRecord, LineIndex, LineSlice, LineSpan, MmapJsonlDataset, SimdJsonlParser,
};
pub use metrics::{
    DatasetCounters, DatasetLatency, DatasetMetricsExporter, DatasetMetricsSnapshot, LatencyTimer,
    NoopMetricsExporter, ThroughputMeter, ThroughputSnapshot,
};
pub use parquet::{
    ArrowDatasetBatch, ParquetDatasetOptions, ParquetDatasetReader, ParquetProjection,
    ParquetTextSchema,
};
pub use pipeline::{ParallelJsonlPipeline, PipelineOptions, PipelineStats, SampleBatch};
pub use plugin::{DatasetPlugin, DATASET_PLUGIN_NAME, DATASET_PLUGIN_VERSION};
pub use reader::{DatasetReader, StreamingJsonlReader};
pub use resume::{
    CheckpointRestorePlan, DatasetCursor, DatasetOffset, DatasetResumeSnapshot, DatasetResumeState,
    RestorePolicy,
};
pub use sampling::{
    ReservoirSampler, SamplingPlan, SamplingSeed, SamplingStrategy, ShardSpec, WeightedSampler,
};
pub use source::TunexDatasetSource;

/// Default dataset configuration type for the JSONL v0.1 backend.
pub type DatasetConfig = JsonlDatasetConfig;
/// Public Parquet dataset reader alias.
pub type ParquetDataset = ParquetDatasetReader;
/// Public filter rule trait alias.
pub use filter::DatasetFilter as FilterRule;
/// Public TuneX dataset plugin alias.
pub type TunexDatasetPlugin = DatasetPlugin;
