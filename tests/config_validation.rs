use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::{CompressionKind, DatasetMode, JsonlDatasetConfig, ValidateDatasetConfig};

fn fixture_path(name: &str, suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("tunex-config-{name}-{nanos}{suffix}"))
}

#[test]
fn valid_jsonl_config_passes_validation() {
    let path = fixture_path("valid", ".jsonl");
    fs::write(&path, "{\"text\":\"ok\"}\n").expect("fixture should write");

    let config = JsonlDatasetConfig::new(&path).with_mmap();

    assert!(config.validate().is_valid());

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn validation_requires_existing_file_and_positive_sizes() {
    let mut config = JsonlDatasetConfig::new("/tmp/tunex-missing-file.jsonl");
    config.pipeline.batch_size = 0;
    config.pipeline.chunk_size_mb = 0;
    config.pipeline.prefetch_batches = 0;
    config.pipeline.workers = 0;

    let report = config.validate();

    assert!(!report.is_valid());
    assert!(report.errors.iter().any(|error| error.path == "path"));
    assert!(report.errors.iter().any(|error| error.path == "batch_size"));
    assert!(report
        .errors
        .iter()
        .any(|error| error.path == "chunk_size_mb"));
    assert!(report
        .errors
        .iter()
        .any(|error| error.path == "prefetch_batches"));
    assert!(report.errors.iter().any(|error| error.path == "workers"));
}

#[test]
fn validation_rejects_compression_extension_mismatch() {
    let path = fixture_path("mismatch", ".jsonl.zst");
    fs::write(&path, b"not decoded").expect("fixture should write");
    let mut config = JsonlDatasetConfig::new(&path);
    config.compression = CompressionKind::Lz4;

    let report = config.validate();

    assert!(report
        .errors
        .iter()
        .any(|error| error.path == "compression"));

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn validation_rejects_mmap_for_compressed_files() {
    let path = fixture_path("mmap-compressed", ".jsonl.lz4");
    fs::write(&path, b"not decoded").expect("fixture should write");
    let mut config = JsonlDatasetConfig::new(&path);
    config.mode = DatasetMode::Mmap;

    let report = config.validate();

    assert!(report.errors.iter().any(|error| error.path == "mmap"));

    fs::remove_file(path).expect("fixture should remove");
}
