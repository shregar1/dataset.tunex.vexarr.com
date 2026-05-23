//! JSONL record contracts.

use bytes::Bytes;

/// Parsed JSONL record ready to become a raw sample.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JsonlRecord {
    /// One-based source line number.
    pub line: u64,
    /// Optional stable record id.
    pub id: Option<String>,
    /// Extracted text payload.
    pub text: Bytes,
}

impl JsonlRecord {
    /// Creates a parsed JSONL record.
    #[must_use]
    pub const fn new(line: u64, id: Option<String>, text: Bytes) -> Self {
        Self { line, id, text }
    }
}
