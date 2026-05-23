//! JSONL dataset reader.

mod decode;
mod fast_path;
mod line_index;
mod mmap_reader;
mod parser;
mod reader;
mod record;
mod simd_parser;
mod slice;

pub use decode::{DecodedJsonlRecord, DecodedText};
pub use fast_path::FastPathJsonlParser;
pub use line_index::{LineIndex, LineSpan};
pub use mmap_reader::MmapJsonlDataset;
pub use parser::JsonlParser;
pub use reader::{JsonlDataset, JsonlDatasetSource};
pub use record::JsonlRecord;
pub use simd_parser::SimdJsonlParser;
pub use slice::LineSlice;
