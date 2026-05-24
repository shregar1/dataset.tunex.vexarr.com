//! Arrow batch helpers for dataset text extraction.

use arrow::{
    array::{Array, ArrayRef, LargeStringArray, StringArray},
    error::ArrowError,
};

use crate::{DatasetError, DatasetResult};

/// Converts an Arrow error into the dataset error model.
#[must_use]
pub fn arrow_error(error: ArrowError) -> DatasetError {
    DatasetError::Arrow(error.to_string())
}

/// Returns a UTF-8 value from a String or LargeString Arrow array.
pub fn string_value(array: &ArrayRef, row: usize, column: &str) -> DatasetResult<Option<String>> {
    if array.is_null(row) {
        return Ok(None);
    }

    if let Some(strings) = array.as_any().downcast_ref::<StringArray>() {
        return Ok(Some(strings.value(row).to_owned()));
    }

    if let Some(strings) = array.as_any().downcast_ref::<LargeStringArray>() {
        return Ok(Some(strings.value(row).to_owned()));
    }

    Err(DatasetError::Arrow(format!(
        "column `{column}` must be Utf8 or LargeUtf8"
    )))
}
