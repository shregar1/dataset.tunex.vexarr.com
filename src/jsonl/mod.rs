//! JSONL dataset reader.

mod parser;
mod reader;
mod record;

pub use parser::JsonlParser;
pub use reader::{JsonlDataset, JsonlDatasetSource};
pub use record::JsonlRecord;
