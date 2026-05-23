//! LZ4 frame input decoder.

use std::{fs::File, io::Read, path::Path};

use lz4_flex::frame::FrameDecoder;

use crate::{DatasetError, DatasetResult};

/// Opens an LZ4 frame decoder.
///
/// # Errors
///
/// Returns an error when the file cannot be opened.
pub fn open(path: &Path) -> DatasetResult<Box<dyn Read + Send>> {
    let file = File::open(path).map_err(|error| DatasetError::io(path, error))?;
    Ok(Box::new(FrameDecoder::new(file)))
}
