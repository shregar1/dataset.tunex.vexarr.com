//! Columnar Arrow batch wrapper for Parquet datasets.

use arrow::record_batch::RecordBatch;
use bytes::Bytes;
use tunex_core::types::RawSample;

use crate::{DatasetError, DatasetResult};

use super::{arrow::string_value, ParquetTextSchema};

/// A projected Arrow record batch plus text-column metadata.
#[derive(Clone, Debug)]
pub struct ArrowDatasetBatch {
    record_batch: RecordBatch,
    text_column_index: usize,
    id_column_index: Option<usize>,
}

impl ArrowDatasetBatch {
    /// Creates a dataset batch from an Arrow record batch.
    pub fn new(record_batch: RecordBatch, text_schema: &ParquetTextSchema) -> DatasetResult<Self> {
        text_schema.validate_arrow_schema(record_batch.schema().as_ref())?;

        let schema = record_batch.schema();
        let text_column_index = schema
            .index_of(&text_schema.text_column)
            .map_err(|error| DatasetError::Arrow(error.to_string()))?;

        let id_column_index = text_schema
            .id_column
            .as_ref()
            .and_then(|id_column| schema.index_of(id_column).ok());

        Ok(Self {
            record_batch,
            text_column_index,
            id_column_index,
        })
    }

    /// Returns the underlying Arrow record batch.
    #[must_use]
    pub fn record_batch(&self) -> &RecordBatch {
        &self.record_batch
    }

    /// Returns the number of rows in the batch.
    #[must_use]
    pub fn num_rows(&self) -> usize {
        self.record_batch.num_rows()
    }

    /// Converts this columnar batch into row-oriented TuneX raw samples.
    pub fn to_samples(&self) -> DatasetResult<Vec<RawSample>> {
        let text_column = self.record_batch.column(self.text_column_index);
        let id_column = self
            .id_column_index
            .map(|index| self.record_batch.column(index));
        let mut samples = Vec::with_capacity(self.record_batch.num_rows());

        for row in 0..self.record_batch.num_rows() {
            let Some(text) = string_value(text_column, row, "text")? else {
                continue;
            };
            let id = id_column
                .map(|column| string_value(column, row, "id"))
                .transpose()?
                .flatten();
            let payload = Bytes::from(text);

            samples.push(match id {
                Some(id) => RawSample::with_id(id, payload),
                None => RawSample::new(payload),
            });
        }

        Ok(samples)
    }
}
