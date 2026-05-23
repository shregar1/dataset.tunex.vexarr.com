//! SIMD-accelerated JSONL parsing.

use bytes::Bytes;
use simd_json::prelude::{ValueAsObject, ValueAsScalar};
use smallvec::SmallVec;

use crate::{DatasetError, DatasetResult, JsonlDatasetConfig};

use super::{DecodedJsonlRecord, DecodedText, FastPathJsonlParser, JsonlRecord};

/// SIMD-accelerated parser for JSONL records.
#[derive(Clone, Debug)]
pub struct SimdJsonlParser {
    text_field: String,
    id_field: Option<String>,
    fast_path: FastPathJsonlParser,
}

impl SimdJsonlParser {
    /// Creates a SIMD parser from dataset config.
    #[must_use]
    pub fn new(config: &JsonlDatasetConfig) -> Self {
        Self {
            text_field: config.text_field.clone(),
            id_field: config.id_field.clone(),
            fast_path: FastPathJsonlParser::new(config.text_field.clone(), config.id_field.clone()),
        }
    }

    /// Parses one shared JSONL line.
    ///
    /// # Errors
    ///
    /// Returns an error when JSON is malformed or the configured text field is
    /// missing.
    pub fn parse_bytes(&self, line: u64, raw: Bytes) -> DatasetResult<JsonlRecord> {
        if let Some(record) = self.fast_path.parse(line, &raw) {
            return Ok(record.into_record());
        }

        self.parse_simd(line, raw)
            .map(DecodedJsonlRecord::into_record)
    }

    /// Parses one mutable JSONL line with SIMD.
    ///
    /// # Errors
    ///
    /// Returns an error when JSON is malformed or the configured text field is
    /// missing.
    pub fn parse_mut(&self, line: u64, raw: &mut [u8]) -> DatasetResult<JsonlRecord> {
        let value =
            simd_json::to_borrowed_value(raw).map_err(|error| DatasetError::InvalidJson {
                line,
                message: error.to_string(),
            })?;
        let object = value.as_object().ok_or_else(|| DatasetError::InvalidJson {
            line,
            message: "expected JSON object".to_owned(),
        })?;
        let text = object
            .get(self.text_field.as_str())
            .and_then(ValueAsScalar::as_str)
            .ok_or_else(|| DatasetError::MissingText {
                line,
                field: self.text_field.clone(),
            })?;
        let id = self
            .id_field
            .as_deref()
            .and_then(|field| object.get(field))
            .and_then(ValueAsScalar::as_str)
            .map(ToOwned::to_owned);

        Ok(JsonlRecord::new(
            line,
            id,
            Bytes::copy_from_slice(text.as_bytes()),
        ))
    }

    /// Parses a batch of shared JSONL lines.
    ///
    /// # Errors
    ///
    /// Returns the first parse error with its source line number.
    pub fn parse_batch<I>(&self, records: I) -> DatasetResult<SmallVec<[JsonlRecord; 32]>>
    where
        I: IntoIterator<Item = (u64, Bytes)>,
    {
        records
            .into_iter()
            .map(|(line, raw)| self.parse_bytes(line, raw))
            .collect()
    }

    fn parse_simd(&self, line: u64, raw: Bytes) -> DatasetResult<DecodedJsonlRecord> {
        let mut owned = raw.to_vec();
        match self.parse_owned_simd(line, &mut owned) {
            Ok(record) => Ok(record),
            Err(simd_error) => self.parse_fallback(line, raw).or(Err(simd_error)),
        }
    }

    fn parse_owned_simd(&self, line: u64, raw: &mut [u8]) -> DatasetResult<DecodedJsonlRecord> {
        let value =
            simd_json::to_borrowed_value(raw).map_err(|error| DatasetError::InvalidJson {
                line,
                message: error.to_string(),
            })?;
        let object = value.as_object().ok_or_else(|| DatasetError::InvalidJson {
            line,
            message: "expected JSON object".to_owned(),
        })?;
        let text = object
            .get(self.text_field.as_str())
            .and_then(ValueAsScalar::as_str)
            .ok_or_else(|| DatasetError::MissingText {
                line,
                field: self.text_field.clone(),
            })?;
        let id = self
            .id_field
            .as_deref()
            .and_then(|field| object.get(field))
            .and_then(ValueAsScalar::as_str)
            .map(ToOwned::to_owned);

        Ok(DecodedJsonlRecord {
            line,
            id,
            text: DecodedText::Owned(Bytes::copy_from_slice(text.as_bytes())),
        })
    }

    fn parse_fallback(&self, line: u64, raw: Bytes) -> DatasetResult<DecodedJsonlRecord> {
        let value: serde_json::Value =
            serde_json::from_slice(raw.as_ref()).map_err(|error| DatasetError::InvalidJson {
                line,
                message: error.to_string(),
            })?;
        let text = value
            .get(&self.text_field)
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| DatasetError::MissingText {
                line,
                field: self.text_field.clone(),
            })?;
        let id = self
            .id_field
            .as_deref()
            .and_then(|field| value.get(field))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned);

        Ok(DecodedJsonlRecord {
            line,
            id,
            text: DecodedText::Owned(Bytes::copy_from_slice(text.as_bytes())),
        })
    }
}
