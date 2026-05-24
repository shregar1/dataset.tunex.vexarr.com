//! Projection contracts for Parquet column selection.

use arrow::datatypes::Schema;

use crate::{DatasetError, DatasetResult};

use super::ParquetTextSchema;

/// Column projection requested from a Parquet dataset.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParquetProjection {
    columns: Vec<String>,
}

impl ParquetProjection {
    /// Creates an empty projection, which reads all columns.
    #[must_use]
    pub fn all() -> Self {
        Self::default()
    }

    /// Creates a projection from column names.
    #[must_use]
    pub fn columns(columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
        }
    }

    /// Creates a minimal projection for text extraction.
    #[must_use]
    pub fn for_text_schema(schema: &ParquetTextSchema) -> Self {
        let mut columns = vec![schema.text_column.clone()];

        if let Some(id_column) = &schema.id_column {
            columns.push(id_column.clone());
        }

        Self { columns }
    }

    /// Returns true when no projection was requested.
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns requested column names.
    #[must_use]
    pub fn columns_ref(&self) -> &[String] {
        &self.columns
    }

    /// Resolves requested columns against an Arrow schema.
    pub fn indices_for_schema(&self, schema: &Schema) -> DatasetResult<Vec<usize>> {
        self.columns
            .iter()
            .map(|column| {
                schema.index_of(column).map_err(|error| {
                    DatasetError::Arrow(format!(
                        "projection column `{column}` is not present in schema: {error}"
                    ))
                })
            })
            .collect()
    }
}
