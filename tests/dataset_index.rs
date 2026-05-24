use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use bytes::Bytes;
use tunex_core::types::RawSample;
use tunex_dataset::{
    CacheMetadata, DatasetIdentity, DatasetManifestRegistry, JsonlDatasetConfig, ManifestBuilder,
    SchemaMetadata, ShardMetadata, SplitMetadata, StatisticsBuilder,
};

#[test]
fn statistics_track_lengths_averages_and_duplicates() {
    let mut builder = StatisticsBuilder::default();
    builder.record_sample(&sample("hello"));
    builder.record_sample(&sample("hello"));
    builder.record_sample(&sample("longer text"));
    builder.record_language("unknown", 3);

    let stats = builder.finish();

    assert_eq!(stats.min_length, 5);
    assert_eq!(stats.max_length, 11);
    assert_eq!(stats.avg_bytes_per_sample, 7.0);
    assert_eq!(stats.duplicate_ratio, 1.0 / 3.0);
    assert_eq!(stats.language_distribution.get("unknown"), Some(&3));
}

#[test]
fn manifest_builder_generates_stable_fingerprint_from_metadata() {
    let stats = StatisticsBuilder::default().finish();
    let first = ManifestBuilder::new(DatasetIdentity::new("train", "v1"), "jsonl")
        .with_sample_count(10)
        .with_byte_size(256)
        .with_statistics(stats.clone())
        .with_schema(SchemaMetadata {
            text_field: "text".to_owned(),
            id_field: Some("id".to_owned()),
            columns: vec!["text".to_owned(), "id".to_owned()],
        })
        .with_split(SplitMetadata {
            name: "train".to_owned(),
            sample_count: 10,
            byte_size: 256,
        })
        .with_shard(ShardMetadata {
            shard_id: 0,
            shard_count: 1,
            sample_count: 10,
        })
        .with_cache(CacheMetadata {
            namespace: "dataset/train/v1".to_owned(),
            parsed_samples: true,
            arrow_batches: false,
        })
        .build();
    let second = ManifestBuilder::new(DatasetIdentity::new("train", "v1"), "jsonl")
        .with_sample_count(10)
        .with_byte_size(256)
        .with_statistics(stats)
        .with_schema(SchemaMetadata {
            text_field: "text".to_owned(),
            id_field: Some("id".to_owned()),
            columns: vec!["text".to_owned(), "id".to_owned()],
        })
        .with_split(SplitMetadata {
            name: "train".to_owned(),
            sample_count: 10,
            byte_size: 256,
        })
        .build();

    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(first.sample_count, 10);
    assert_eq!(first.splits[0].name, "train");
    assert!(first.cache.expect("cache").parsed_samples);
}

#[test]
fn manifest_from_jsonl_config_captures_file_and_schema_metadata() {
    let dir = std::env::temp_dir().join(format!(
        "tunex-index-test-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("dir");
    let path = dir.join("train.jsonl");
    fs::write(&path, "{\"text\":\"hello\"}\n").expect("write");

    let config = JsonlDatasetConfig::new(&path);
    let manifest = ManifestBuilder::from_jsonl_config("train", "v1", &config)
        .expect("builder")
        .with_sample_count(1)
        .build();

    assert_eq!(manifest.identity.name, "train");
    assert_eq!(manifest.byte_size, 17);
    assert_eq!(manifest.compression.format, "plain");
    assert_eq!(manifest.schema.text_field, "text");
    assert_eq!(manifest.schema.id_field.as_deref(), Some("id"));

    fs::remove_dir_all(dir).expect("cleanup");
}

#[test]
fn manifest_registry_tracks_manifests_by_fingerprint() {
    let manifest = ManifestBuilder::new(DatasetIdentity::new("validation", "v1"), "jsonl")
        .with_sample_count(2)
        .build();
    let fingerprint = manifest.fingerprint.clone();
    let mut registry = DatasetManifestRegistry::default();

    registry.register(manifest);

    assert_eq!(registry.list().len(), 1);
    assert_eq!(
        registry.get(&fingerprint).expect("manifest").sample_count,
        2
    );
    assert!(registry.remove(&fingerprint).is_some());
    assert!(registry.get(&fingerprint).is_none());
}

fn sample(text: &'static str) -> RawSample {
    RawSample::new(Bytes::from_static(text.as_bytes()))
}
