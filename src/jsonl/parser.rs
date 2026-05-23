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
        let value: Value =
            serde_json::from_slice(raw).map_err(|error| DatasetError::InvalidJson {
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

        Ok(JsonlRecord::new(
            line,
            id,
            Bytes::copy_from_slice(text.as_bytes()),
        ))
    }
}
