//! Compression reader helpers.

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use lz4_flex::frame::FrameDecoder as Lz4Decoder;
use zstd::stream::read::Decoder as ZstdDecoder;

use crate::{CompressionKind, DatasetError, DatasetResult};

/// Opens a file as a reader using the selected compression format.
///
/// # Errors
///
/// Returns an error when the file cannot be opened or the decoder cannot be
/// initialized.
pub fn open_reader(
    path: &Path,
    compression: CompressionKind,
) -> DatasetResult<Box<dyn Read + Send>> {
    let file = File::open(path).map_err(|error| DatasetError::io(path, error))?;
    let reader: Box<dyn Read + Send> = match compression {
        CompressionKind::None => Box::new(BufReader::new(file)),
        CompressionKind::Zstd => {
            Box::new(ZstdDecoder::new(file).map_err(|error| DatasetError::io(path, error))?)
        }
        CompressionKind::Lz4 => Box::new(Lz4Decoder::new(file)),
    };
    Ok(reader)
}
