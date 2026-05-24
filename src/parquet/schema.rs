//! Schema contracts for text-oriented Parquet datasets.

use arrow::datatypes::{DataType, Schema};

use crate::{DatasetError, DatasetResult};

/// Text and optional identifier columns expected by the dataset reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParquetTextSchema {
    /// Column containing raw training text.
    pub text_column: String,
    /// Optional column containing stable sample identifiers.
    pub id_column: Option<String>,
}

impl ParquetTextSchema {
    /// Creates a schema contract for a required text column.
    #[must_use]
    pub fn new(text_column: impl Into<String>) -> Self {
        Self {
            text_column: text_column.into(),
            id_column: None,
        }
    }

    /// Sets the optional identifier column.
    #[must_use]
    pub fn with_id_column(mut self, id_column: impl Into<String>) -> Self {
        self.id_column = Some(id_column.into());
        self
    }

    /// Validates that required columns exist and are UTF-8 compatible.
    pub fn validate_arrow_schema(&self, schema: &Schema) -> DatasetResult<()> {
        validate_utf8_column(schema, &self.text_column)?;

        if let Some(id_column) = &self.id_column {
            validate_utf8_column(schema, id_column)?;
        }

        Ok(())
    }
}

impl Default for ParquetTextSchema {
    fn default() -> Self {
        Self::new("text").with_id_column("id")
    }
}

fn validate_utf8_column(schema: &Schema, column: &str) -> DatasetResult<()> {
    let field = schema.field_with_name(column).map_err(|error| {
        DatasetError::Arrow(format!("missing required column `{column}`: {error}"))
    })?;

    match field.data_type() {
        DataType::Utf8 | DataType::LargeUtf8 => Ok(()),
        data_type => Err(DatasetError::Arrow(format!(
            "column `{column}` must be Utf8 or LargeUtf8, got {data_type:?}"
        ))),
    }
}
