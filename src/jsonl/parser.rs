//! JSONL parsing compatibility wrapper.

use bytes::Bytes;

use crate::{DatasetResult, JsonlDatasetConfig};

use super::{JsonlRecord, SimdJsonlParser};

/// Parser for one JSONL row.
#[derive(Clone, Debug)]
pub struct JsonlParser {
    parser: SimdJsonlParser,
}

impl JsonlParser {
    /// Creates a parser from dataset config.
    #[must_use]
    pub fn new(config: &JsonlDatasetConfig) -> Self {
        Self {
            parser: SimdJsonlParser::new(config),
        }
    }

    /// Parses one JSONL line into a record.
    ///
    /// # Errors
    ///
    /// Returns an error when the row is malformed JSON or does not contain the
    /// configured string text field.
    pub fn parse_line(&self, line: u64, raw: &[u8]) -> DatasetResult<JsonlRecord> {
        self.parse_bytes(line, Bytes::copy_from_slice(raw))
    }

    /// Parses one JSONL line from shared bytes into a record.
    ///
    /// # Errors
    ///
    /// Returns an error when the row is malformed JSON or does not contain the
    /// configured string text field.
    pub fn parse_bytes(&self, line: u64, raw: Bytes) -> DatasetResult<JsonlRecord> {
        self.parser.parse_bytes(line, raw)
    }
}
