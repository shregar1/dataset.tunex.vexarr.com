//! Dataset manifest metadata.

/// Stable dataset identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetIdentity {
    /// Dataset name.
    pub name: String,
    /// Dataset version.
    pub version: String,
}

impl DatasetIdentity {
    /// Creates dataset identity metadata.
    #[must_use]
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

/// Schema metadata for the source dataset.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchemaMetadata {
    /// Text column or field name.
    pub text_field: String,
    /// Optional id column or field name.
    pub id_field: Option<String>,
    /// Additional column names.
    pub columns: Vec<String>,
}

/// Split metadata such as train/validation/test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SplitMetadata {
    /// Split name.
    pub name: String,
    /// Sample count in the split.
    pub sample_count: u64,
    /// Byte size in the split.
    pub byte_size: u64,
}

/// Compression metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompressionMetadata {
    /// Compression format label.
    pub format: String,
    /// Compressed size in bytes, when known.
    pub compressed_bytes: Option<u64>,
    /// Uncompressed size in bytes, when known.
    pub uncompressed_bytes: Option<u64>,
}

/// Distributed shard metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShardMetadata {
    /// Shard id or rank.
    pub shard_id: u32,
    /// Total shard count.
    pub shard_count: u32,
    /// Sample count owned by this shard.
    pub sample_count: u64,
}

/// Cache metadata linked to this dataset identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheMetadata {
    /// Cache namespace or key prefix.
    pub namespace: String,
    /// Whether parsed sample cache is available.
    pub parsed_samples: bool,
    /// Whether Arrow batch cache is available.
    pub arrow_batches: bool,
}
