//! Dataset loading engine for the `TuneX` ecosystem.
//!
//! The v0.1 surface focuses on JSONL records that can be read through bounded
//! streaming or memory-mapped readers and emitted as `tunex-core` `RawSample`
//! values through the `DatasetSource` contract.

pub mod batch;
pub mod compression;
pub mod config;
pub mod error;
pub mod jsonl;
pub mod mmap;
pub mod pipeline;
pub mod plugin;
pub mod reader;
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
pub use error::{DatasetError, DatasetResult};
pub use jsonl::{
    DecodedJsonlRecord, DecodedText, FastPathJsonlParser, JsonlDataset, JsonlDatasetSource,
    JsonlParser, JsonlRecord, LineIndex, LineSlice, LineSpan, MmapJsonlDataset, SimdJsonlParser,
};
pub use pipeline::{ParallelJsonlPipeline, PipelineOptions, PipelineStats, SampleBatch};
pub use plugin::{DatasetPlugin, DATASET_PLUGIN_NAME, DATASET_PLUGIN_VERSION};
pub use reader::{DatasetReader, StreamingJsonlReader};
pub use sampling::{
    ReservoirSampler, SamplingPlan, SamplingSeed, SamplingStrategy, ShardSpec, WeightedSampler,
};
pub use source::TunexDatasetSource;
