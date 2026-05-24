//! Arrow and Parquet dataset backend.
//!
//! This module keeps Parquet reads columnar by default. Callers can consume
//! [`ArrowDatasetBatch`] values directly, or explicitly convert text columns
//! into `RawSample` values at the boundary where row-oriented samples are
//! required.

pub mod arrow;
pub mod batch;
pub mod projection;
pub mod reader;
pub mod schema;

pub use arrow::{arrow_error, string_value};
pub use batch::ArrowDatasetBatch;
pub use projection::ParquetProjection;
pub use reader::{ParquetDatasetOptions, ParquetDatasetReader};
pub use schema::ParquetTextSchema;
