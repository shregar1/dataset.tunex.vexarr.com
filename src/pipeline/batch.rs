//! Pipeline batch types.

use tunex_core::types::RawSample;

/// One raw JSONL line scheduled for parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawLine {
    /// One-based source line number.
    pub line: u64,
    /// Raw line bytes after newline trimming.
    pub bytes: Vec<u8>,
}

/// A cache-friendly chunk of lines.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LineChunk {
    /// Lines in this chunk.
    pub lines: Vec<RawLine>,
    /// Total raw bytes in the chunk.
    pub bytes: usize,
}

impl LineChunk {
    /// Creates an empty chunk with capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            lines: Vec::with_capacity(capacity),
            bytes: 0,
        }
    }

    /// Adds one line to the chunk.
    pub fn push(&mut self, line: RawLine) {
        self.bytes = self.bytes.saturating_add(line.bytes.len());
        self.lines.push(line);
    }

    /// Returns true when this chunk has no lines.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

/// Parsed sample batch emitted by the pipeline.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SampleBatch {
    /// Parsed samples.
    pub samples: Vec<RawSample>,
    /// Raw bytes represented by the batch.
    pub bytes: usize,
    /// First source line in the batch.
    pub first_line: Option<u64>,
    /// Last source line in the batch.
    pub last_line: Option<u64>,
}

impl SampleBatch {
    /// Creates a sample batch.
    #[must_use]
    pub fn new(
        samples: Vec<RawSample>,
        bytes: usize,
        first_line: Option<u64>,
        last_line: Option<u64>,
    ) -> Self {
        Self {
            samples,
            bytes,
            first_line,
            last_line,
        }
    }

    /// Returns number of samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Returns true when the batch is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}
