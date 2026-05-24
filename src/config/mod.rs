//! Dataset loader configuration and validation.

pub mod compression;
pub mod jsonl;
pub mod pipeline;
pub mod validation;

pub use compression::CompressionKind;
pub use jsonl::{DatasetFormat, DatasetMode, JsonlDatasetConfig};
pub use pipeline::DatasetPipelineConfig;
pub use validation::{ConfigValidationError, ConfigValidationReport, ValidateDatasetConfig};
