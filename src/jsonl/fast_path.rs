//! Fast-path parser for simple JSONL rows.

use bytes::Bytes;

use super::{DecodedJsonlRecord, DecodedText};

/// Fast-path parser for rows shaped like `{"text":"..."}`.
#[derive(Clone, Debug)]
pub struct FastPathJsonlParser {
    text_field: String,
    id_field: Option<String>,
}

impl FastPathJsonlParser {
    /// Creates a fast-path parser.
    #[must_use]
    pub fn new(text_field: impl Into<String>, id_field: Option<String>) -> Self {
        Self {
            text_field: text_field.into(),
            id_field,
        }
    }

    /// Parses simple top-level string fields without allocating text.
    #[must_use]
    pub fn parse(&self, line: u64, raw: &Bytes) -> Option<DecodedJsonlRecord> {
        let bytes = raw.as_ref();
        let mut cursor = skip_ws(bytes, 0);
        if bytes.get(cursor) != Some(&b'{') {
            return None;
        }
        cursor += 1;

        let mut id = None;
        let mut text = None;

        loop {
            cursor = skip_ws(bytes, cursor);
            match bytes.get(cursor) {
                Some(b'}') => {
                    cursor = skip_ws(bytes, cursor + 1);
                    if cursor != bytes.len() {
                        return None;
                    }
                    let text = text?;
                    return Some(DecodedJsonlRecord { line, id, text });
                }
                Some(b'"') => {}
                _ => return None,
            }

            let key = parse_unescaped_string(bytes, cursor)?;
            cursor = skip_ws(bytes, key.end + 1);
            if bytes.get(cursor) != Some(&b':') {
                return None;
            }
            cursor = skip_ws(bytes, cursor + 1);
            if bytes.get(cursor) != Some(&b'"') {
                return None;
            }

            let value = parse_unescaped_string(bytes, cursor)?;
            if key.as_bytes(bytes) == self.text_field.as_bytes() {
                text = Some(DecodedText::Borrowed(raw.slice(value.start..value.end)));
            } else if self
                .id_field
                .as_deref()
                .is_some_and(|field| key.as_bytes(bytes) == field.as_bytes())
            {
                id = Some(String::from_utf8_lossy(value.as_bytes(bytes)).into_owned());
            }

            cursor = skip_ws(bytes, value.end + 1);
            match bytes.get(cursor) {
                Some(b',') => cursor += 1,
                Some(b'}') => {}
                _ => return None,
            }
        }
    }
}

#[derive(Clone, Copy)]
struct StringSpan {
    start: usize,
    end: usize,
}

impl StringSpan {
    fn as_bytes(self, bytes: &[u8]) -> &[u8] {
        &bytes[self.start..self.end]
    }
}

fn parse_unescaped_string(bytes: &[u8], quote: usize) -> Option<StringSpan> {
    if bytes.get(quote) != Some(&b'"') {
        return None;
    }

    let start = quote + 1;
    let mut cursor = start;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => return None,
            b'"' => return Some(StringSpan { start, end: cursor }),
            _ => cursor += 1,
        }
    }
    None
}

fn skip_ws(bytes: &[u8], mut cursor: usize) -> usize {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        cursor += 1;
    }
    cursor
}
