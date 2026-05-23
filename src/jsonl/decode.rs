//! Decoded JSONL row primitives.

use bytes::Bytes;

use super::JsonlRecord;

/// Text payload decoded from one JSONL row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecodedText {
    /// Text borrowed from the original shared line bytes.
    Borrowed(Bytes),
    /// Text materialized after unescaping or fallback decoding.
    Owned(Bytes),
}

impl DecodedText {
    /// Returns the decoded text as shared bytes.
    #[must_use]
    pub fn into_bytes(self) -> Bytes {
        match self {
            Self::Borrowed(bytes) | Self::Owned(bytes) => bytes,
        }
    }

    /// Returns true when text was borrowed from the original line buffer.
    #[must_use]
    pub const fn is_borrowed(&self) -> bool {
        matches!(self, Self::Borrowed(_))
    }
}

/// Decoded record before conversion into the public `JsonlRecord`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedJsonlRecord {
    /// One-based source line number.
    pub line: u64,
    /// Optional stable record id.
    pub id: Option<String>,
    /// Decoded text payload.
    pub text: DecodedText,
}

impl DecodedJsonlRecord {
    /// Converts this decoded row into the public record type.
    #[must_use]
    pub fn into_record(self) -> JsonlRecord {
        JsonlRecord::new(self.line, self.id, self.text.into_bytes())
    }
}
