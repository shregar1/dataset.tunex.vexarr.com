//! Dataset manifest generation.

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    compression::CompressionFormat, DatasetError, DatasetFormat, DatasetResult, JsonlDatasetConfig,
};

use super::{
    CacheMetadata, CompressionMetadata, DatasetFingerprint, DatasetIdentity, DatasetStatistics,
    FingerprintBuilder, SchemaMetadata, ShardMetadata, SplitMetadata,
};

/// Dataset manifest used as the metadata/control plane.
#[derive(Clone, Debug, PartialEq)]
pub struct DatasetManifest {
    /// Dataset identity.
    pub identity: DatasetIdentity,
    /// Format label.
    pub format: String,
    /// Total sample count.
    pub sample_count: u64,
    /// Total byte size.
    pub byte_size: u64,
    /// Compression metadata.
    pub compression: CompressionMetadata,
    /// Dataset splits.
    pub splits: Vec<SplitMetadata>,
    /// Stable dataset fingerprint.
    pub fingerprint: DatasetFingerprint,
    /// Creation timestamp as Unix seconds.
    pub created_at_unix: u64,
    /// Dataset statistics.
    pub statistics: DatasetStatistics,
    /// Schema metadata.
    pub schema: SchemaMetadata,
    /// Distributed shard metadata.
    pub shards: Vec<ShardMetadata>,
    /// Cache metadata.
    pub cache: Option<CacheMetadata>,
}

/// Builder for dataset manifests.
#[derive(Clone, Debug)]
pub struct ManifestBuilder {
    identity: DatasetIdentity,
    format: String,
    sample_count: u64,
    byte_size: u64,
    compression: CompressionMetadata,
    splits: Vec<SplitMetadata>,
    statistics: DatasetStatistics,
    schema: SchemaMetadata,
    shards: Vec<ShardMetadata>,
    cache: Option<CacheMetadata>,
}

impl ManifestBuilder {
    /// Creates a manifest builder.
    #[must_use]
    pub fn new(identity: DatasetIdentity, format: impl Into<String>) -> Self {
        Self {
            identity,
            format: format.into(),
            sample_count: 0,
            byte_size: 0,
            compression: CompressionMetadata {
                format: "plain".to_owned(),
                compressed_bytes: None,
                uncompressed_bytes: None,
            },
            splits: Vec::new(),
            statistics: DatasetStatistics::default(),
            schema: SchemaMetadata::default(),
            shards: Vec::new(),
            cache: None,
        }
    }

    /// Creates a builder from a JSONL config and file metadata.
    pub fn from_jsonl_config(
        name: impl Into<String>,
        version: impl Into<String>,
        config: &JsonlDatasetConfig,
    ) -> DatasetResult<Self> {
        let metadata =
            fs::metadata(&config.path).map_err(|source| DatasetError::io(&config.path, source))?;
        let compression = CompressionFormat::resolve(&config.path, config.compression);
        Ok(Self::new(
            DatasetIdentity::new(name, version),
            format!("{:?}", DatasetFormat::Jsonl).to_lowercase(),
        )
        .with_byte_size(metadata.len())
        .with_compression(CompressionMetadata {
            format: format!("{compression:?}").to_lowercase(),
            compressed_bytes: Some(metadata.len()),
            uncompressed_bytes: compression.supports_mmap().then_some(metadata.len()),
        })
        .with_schema(SchemaMetadata {
            text_field: config.text_field.clone(),
            id_field: config.id_field.clone(),
            columns: Vec::new(),
        }))
    }

    /// Sets sample count.
    #[must_use]
    pub const fn with_sample_count(mut self, sample_count: u64) -> Self {
        self.sample_count = sample_count;
        self
    }

    /// Sets byte size.
    #[must_use]
    pub const fn with_byte_size(mut self, byte_size: u64) -> Self {
        self.byte_size = byte_size;
        self
    }

    /// Sets compression metadata.
    #[must_use]
    pub fn with_compression(mut self, compression: CompressionMetadata) -> Self {
        self.compression = compression;
        self
    }

    /// Sets statistics.
    #[must_use]
    pub fn with_statistics(mut self, statistics: DatasetStatistics) -> Self {
        self.statistics = statistics;
        self
    }

    /// Sets schema metadata.
    #[must_use]
    pub fn with_schema(mut self, schema: SchemaMetadata) -> Self {
        self.schema = schema;
        self
    }

    /// Adds split metadata.
    #[must_use]
    pub fn with_split(mut self, split: SplitMetadata) -> Self {
        self.splits.push(split);
        self
    }

    /// Adds shard metadata.
    #[must_use]
    pub fn with_shard(mut self, shard: ShardMetadata) -> Self {
        self.shards.push(shard);
        self
    }

    /// Sets cache metadata.
    #[must_use]
    pub fn with_cache(mut self, cache: CacheMetadata) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Builds the manifest.
    #[must_use]
    pub fn build(self) -> DatasetManifest {
        let fingerprint = self.fingerprint();
        DatasetManifest {
            identity: self.identity,
            format: self.format,
            sample_count: self.sample_count,
            byte_size: self.byte_size,
            compression: self.compression,
            splits: self.splits,
            fingerprint,
            created_at_unix: now_unix_seconds(),
            statistics: self.statistics,
            schema: self.schema,
            shards: self.shards,
            cache: self.cache,
        }
    }

    fn fingerprint(&self) -> DatasetFingerprint {
        let mut builder = FingerprintBuilder::new();
        builder.update_str(&self.identity.name);
        builder.update_str(&self.identity.version);
        builder.update_str(&self.format);
        builder.update_str(&self.sample_count.to_string());
        builder.update_str(&self.byte_size.to_string());
        builder.update_str(&self.compression.format);
        builder.update_str(&self.schema.text_field);
        if let Some(id_field) = &self.schema.id_field {
            builder.update_str(id_field);
        }
        for split in &self.splits {
            builder.update_str(&split.name);
            builder.update_str(&split.sample_count.to_string());
            builder.update_str(&split.byte_size.to_string());
        }
        builder.finish()
    }
}

impl DatasetManifest {
    /// Creates a manifest for a single source path without sample statistics.
    pub fn from_path(
        name: impl Into<String>,
        version: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> DatasetResult<Self> {
        let path = path.as_ref();
        let metadata = fs::metadata(path).map_err(|source| DatasetError::io(path, source))?;
        Ok(
            ManifestBuilder::new(DatasetIdentity::new(name, version), "unknown")
                .with_byte_size(metadata.len())
                .build(),
        )
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
