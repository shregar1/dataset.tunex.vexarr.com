//! Compression input support for dataset readers.

pub mod format;
pub mod lz4;
pub mod reader;
pub mod zstd;

pub use format::CompressionFormat;
pub use reader::{CompressionStats, DatasetInputReader};

use std::{io::Read, path::Path};

use crate::{CompressionKind, DatasetResult};

/// Opens a file as a plain or decompressed stream.
///
/// # Errors
///
/// Returns an error when the file cannot be opened or the decoder cannot be
/// initialized.
pub fn open_reader(
    path: &Path,
    compression: CompressionKind,
) -> DatasetResult<Box<dyn Read + Send>> {
    Ok(DatasetInputReader::open(path, compression)?.into_reader())
}
