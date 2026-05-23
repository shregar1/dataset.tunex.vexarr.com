//! JSONL parsing.

use bytes::Bytes;
use serde_json::Value;

use crate::{DatasetError, DatasetResult, JsonlDatasetConfig};

use super::JsonlRecord;

/// Parser for one JSONL row.
#[derive(Clone, Debug)]
pub struct JsonlParser {
    text_field: String,
    id_field: Option<String>,
}

impl JsonlParser {
    /// Creates a parser from dataset config.
    #[must_use]
    pub fn new(config: &JsonlDatasetConfig) -> Self {
        Self {
            text_field: config.text_field.clone(),
            id_field: config.id_field.clone(),
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
        let value: Value =
            serde_json::from_slice(raw.as_ref()).map_err(|error| DatasetError::InvalidJson {
                line,
                message: error.to_string(),
            })?;
        let text = value
            .get(&self.text_field)
            .and_then(Value::as_str)
            .ok_or_else(|| DatasetError::MissingText {
                line,
                field: self.text_field.clone(),
            })?;
        let id = self
            .id_field
            .as_deref()
            .and_then(|field| value.get(field))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let text_bytes = find_unescaped_string_value(&raw, &self.text_field, text)
            .unwrap_or_else(|| Bytes::copy_from_slice(text.as_bytes()));

        Ok(JsonlRecord::new(line, id, text_bytes))
    }
}

fn find_unescaped_string_value(raw: &Bytes, field: &str, decoded_text: &str) -> Option<Bytes> {
    let key = serde_json::to_vec(field).ok()?;
    let bytes = raw.as_ref();
    let mut search_at = 0;

    while search_at < bytes.len() {
        let key_start = find_subslice(&bytes[search_at..], &key)? + search_at;
        let mut cursor = key_start + key.len();
        cursor = skip_ws(bytes, cursor);
        if bytes.get(cursor) != Some(&b':') {
            search_at = key_start + 1;
            continue;
        }
        cursor = skip_ws(bytes, cursor + 1);
        if bytes.get(cursor) != Some(&b'"') {
            search_at = key_start + 1;
            continue;
        }

        let value_start = cursor + 1;
        cursor = value_start;
        while cursor < bytes.len() {
            match bytes[cursor] {
                b'\\' => return None,
                b'"' => {
                    let candidate = raw.slice(value_start..cursor);
                    if candidate.as_ref() == decoded_text.as_bytes() {
                        return Some(candidate);
                    }
                    search_at = key_start + 1;
                    break;
                }
                _ => cursor += 1,
            }
        }
    }

    None
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn skip_ws(bytes: &[u8], mut cursor: usize) -> usize {
    while matches!(bytes.get(cursor), Some(b' ' | b'\t' | b'\r' | b'\n')) {
        cursor += 1;
    }
    cursor
}
