//! Dataset loading engine for the `TuneX` ecosystem.
//!
//! The v0.1 surface focuses on JSONL records that can be read through bounded
//! streaming or memory-mapped readers and emitted as `tunex-core` `RawSample`
//! values through the `DatasetSource` contract.

pub mod compression;
pub mod config;
pub mod error;
pub mod jsonl;
pub mod mmap;
pub mod plugin;
pub mod reader;

pub use compression::{CompressionFormat, CompressionStats, DatasetInputReader};
pub use config::{CompressionKind, DatasetMode, JsonlDatasetConfig};
pub use error::{DatasetError, DatasetResult};
pub use jsonl::{
    DecodedJsonlRecord, DecodedText, FastPathJsonlParser, JsonlDataset, JsonlDatasetSource,
    JsonlParser, JsonlRecord, LineIndex, LineSlice, LineSpan, MmapJsonlDataset, SimdJsonlParser,
};
pub use plugin::DatasetPlugin;
pub use reader::{DatasetReader, StreamingJsonlReader};
