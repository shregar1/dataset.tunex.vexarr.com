//! Streaming Parquet reader producing Arrow dataset batches.

use std::{fs::File, path::Path};

use parquet::{
    arrow::{
        arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder},
        ProjectionMask,
    },
    errors::ParquetError,
};

use crate::{DatasetError, DatasetResult};

use super::{arrow::arrow_error, ArrowDatasetBatch, ParquetProjection, ParquetTextSchema};

/// Options for reading Parquet-backed datasets.
#[derive(Clone, Debug)]
pub struct ParquetDatasetOptions {
    /// Target Arrow record batch size.
    pub batch_size: usize,
    /// Column projection to push into Parquet decoding.
    pub projection: ParquetProjection,
    /// Text schema used when converting Arrow batches to raw samples.
    pub text_schema: ParquetTextSchema,
    /// Enables future parallel row-group strategies.
    pub parallel_row_groups: bool,
}

impl ParquetDatasetOptions {
    /// Creates options optimized for text extraction.
    #[must_use]
    pub fn text_only() -> Self {
        let text_schema = ParquetTextSchema::default();
        Self {
            batch_size: 8192,
            projection: ParquetProjection::for_text_schema(&text_schema),
            text_schema,
            parallel_row_groups: true,
        }
    }
}

impl Default for ParquetDatasetOptions {
    fn default() -> Self {
        Self::text_only()
    }
}

/// A streaming Parquet dataset reader.
pub struct ParquetDatasetReader {
    reader: ParquetRecordBatchReader,
    text_schema: ParquetTextSchema,
}

impl ParquetDatasetReader {
    /// Opens a Parquet file and prepares a projected Arrow batch reader.
    pub fn open(path: impl AsRef<Path>, options: ParquetDatasetOptions) -> DatasetResult<Self> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| DatasetError::io(path, source))?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file).map_err(parquet_error)?;
        let projection = if options.projection.is_all() {
            ProjectionMask::all()
        } else {
            let indices = options
                .projection
                .indices_for_schema(builder.schema().as_ref())?;
            ProjectionMask::leaves(builder.parquet_schema(), indices)
        };
        let reader = builder
            .with_batch_size(options.batch_size)
            .with_projection(projection)
            .build()
            .map_err(parquet_error)?;

        Ok(Self {
            reader,
            text_schema: options.text_schema,
        })
    }

    /// Reads the next Arrow-backed dataset batch.
    pub fn next_arrow_batch(&mut self) -> DatasetResult<Option<ArrowDatasetBatch>> {
        let Some(batch) = self.reader.next() else {
            return Ok(None);
        };

        ArrowDatasetBatch::new(batch.map_err(arrow_error)?, &self.text_schema).map(Some)
    }

    /// Reads and converts the next batch into raw samples.
    pub fn next_samples(&mut self) -> DatasetResult<Option<Vec<tunex_core::types::RawSample>>> {
        self.next_arrow_batch()?
            .map(|batch| batch.to_samples())
            .transpose()
    }
}

#[must_use]
fn parquet_error(error: ParquetError) -> DatasetError {
    DatasetError::Parquet(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{
        fs::{self, File},
        sync::Arc,
    };

    use arrow::{
        array::{ArrayRef, StringArray},
        datatypes::{DataType, Field, Schema},
        record_batch::RecordBatch,
    };
    use parquet::arrow::ArrowWriter;

    use super::*;

    #[test]
    fn reads_text_samples_from_projected_parquet() {
        let dir = std::env::temp_dir().join(format!(
            "tunex-parquet-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("tempdir");
        let path = dir.join("train.parquet");
        write_test_parquet(&path);

        let mut reader =
            ParquetDatasetReader::open(&path, ParquetDatasetOptions::default()).expect("reader");
        let samples = reader
            .next_samples()
            .expect("samples")
            .expect("first batch");

        assert_eq!(samples.len(), 2);
        assert_eq!(samples[0].payload.as_ref(), b"hello");
        assert_eq!(samples[1].id.as_deref(), Some("b"));
        fs::remove_dir_all(&dir).expect("cleanup");
    }

    fn write_test_parquet(path: &Path) {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new("ignored", DataType::Utf8, false),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec!["a", "b"])) as ArrayRef,
                Arc::new(StringArray::from(vec!["hello", "world"])) as ArrayRef,
                Arc::new(StringArray::from(vec!["x", "y"])) as ArrayRef,
            ],
        )
        .expect("batch");
        let file = File::create(path).expect("file");
        let mut writer = ArrowWriter::try_new(file, schema, None).expect("writer");
        writer.write(&batch).expect("write");
        writer.close().expect("close");
    }
}
