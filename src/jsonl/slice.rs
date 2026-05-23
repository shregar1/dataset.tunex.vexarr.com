//! Borrowed JSONL line slices.

use bytes::Bytes;

/// Zero-copy view of one indexed JSONL line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineSlice {
    /// One-based source line number.
    pub line: u64,
    /// Shared bytes for the line.
    pub bytes: Bytes,
}

impl LineSlice {
    /// Creates a line slice.
    #[must_use]
    pub const fn new(line: u64, bytes: Bytes) -> Self {
        Self { line, bytes }
    }

    /// Returns the line bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_ref()
    }
}
