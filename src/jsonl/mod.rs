//! JSONL dataset reader.

mod line_index;
mod mmap_reader;
mod parser;
mod reader;
mod record;
mod slice;

pub use line_index::{LineIndex, LineSpan};
pub use mmap_reader::MmapJsonlDataset;
pub use parser::JsonlParser;
pub use reader::{JsonlDataset, JsonlDatasetSource};
pub use record::JsonlRecord;
pub use slice::LineSlice;
