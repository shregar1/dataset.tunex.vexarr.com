//! Source offset tracking.

/// Physical source offset for seekable and line-oriented datasets.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DatasetOffset {
    /// Source line number for JSONL-style inputs.
    pub line_number: Option<u64>,
    /// Source byte offset for seekable files.
    pub byte_offset: Option<u64>,
}

impl DatasetOffset {
    /// Creates an empty offset.
    #[must_use]
    pub const fn new(line_number: Option<u64>, byte_offset: Option<u64>) -> Self {
        Self {
            line_number,
            byte_offset,
        }
    }

    /// Creates a JSONL line offset.
    #[must_use]
    pub const fn line(line_number: u64) -> Self {
        Self::new(Some(line_number), None)
    }

    /// Creates a byte offset.
    #[must_use]
    pub const fn byte(byte_offset: u64) -> Self {
        Self::new(None, Some(byte_offset))
    }
}
