//! Zstandard input decoder.

use std::{fs::File, io::Read, path::Path};

use zstd::stream::read::Decoder;

use crate::{DatasetError, DatasetResult};

/// Opens a Zstandard decoder.
///
/// # Errors
///
/// Returns an error when the file or decoder cannot be opened.
pub fn open(path: &Path) -> DatasetResult<Box<dyn Read + Send>> {
    let file = File::open(path).map_err(|error| DatasetError::io(path, error))?;
    let decoder = Decoder::new(file).map_err(|error| DatasetError::io(path, error))?;
    Ok(Box::new(decoder))
}
