//! Line offset index for memory-mapped JSONL files.

/// Byte span for one non-empty JSONL line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineSpan {
    /// One-based source line number.
    pub line: u64,
    /// Inclusive byte start.
    pub start: usize,
    /// Exclusive byte end after newline trimming.
    pub end: usize,
}

/// Newline index over a JSONL byte buffer.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LineIndex {
    spans: Vec<LineSpan>,
}

impl LineIndex {
    /// Builds an index of non-empty JSONL lines.
    #[must_use]
    pub fn build(bytes: &[u8]) -> Self {
        let mut spans = Vec::new();
        let mut start = 0;
        let mut line = 1;

        for (offset, byte) in bytes.iter().enumerate() {
            if *byte == b'\n' {
                push_span(&mut spans, bytes, line, start, offset);
                start = offset.saturating_add(1);
                line += 1;
            }
        }

        if start < bytes.len() {
            push_span(&mut spans, bytes, line, start, bytes.len());
        }

        Self { spans }
    }

    /// Returns the number of indexed non-empty lines.
    #[must_use]
    pub fn len(&self) -> usize {
        self.spans.len()
    }

    /// Returns true if the index has no records.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Returns a line span by record index.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<LineSpan> {
        self.spans.get(index).copied()
    }
}

fn push_span(spans: &mut Vec<LineSpan>, bytes: &[u8], line: u64, start: usize, end: usize) {
    let start = trim_start(bytes, start, end);
    let end = trim_end(bytes, start, end);
    if start < end {
        spans.push(LineSpan { line, start, end });
    }
}

fn trim_start(bytes: &[u8], mut start: usize, end: usize) -> usize {
    while start < end && matches!(bytes[start], b' ' | b'\t' | b'\r') {
        start += 1;
    }
    start
}

fn trim_end(bytes: &[u8], start: usize, mut end: usize) -> usize {
    while end > start && matches!(bytes[end - 1], b' ' | b'\t' | b'\r') {
        end -= 1;
    }
    end
}
