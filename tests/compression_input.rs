use std::{
    fs,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use lz4_flex::frame::FrameEncoder;
use tunex_dataset::{
    CompressionFormat, CompressionKind, DatasetError, JsonlDataset, JsonlDatasetConfig,
    MmapJsonlDataset,
};

fn fixture_path(name: &str, suffix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("tunex-dataset-{name}-{nanos}{suffix}"))
}

#[test]
fn compression_format_detects_jsonl_extensions() {
    assert_eq!(
        CompressionFormat::detect(std::path::Path::new("train.jsonl")),
        CompressionFormat::Plain
    );
    assert_eq!(
        CompressionFormat::detect(std::path::Path::new("train.jsonl.zst")),
        CompressionFormat::Zstd
    );
    assert_eq!(
        CompressionFormat::detect(std::path::Path::new("train.jsonl.lz4")),
        CompressionFormat::Lz4
    );
}

#[test]
fn jsonl_dataset_streams_zstd_input() {
    let path = fixture_path("zstd", ".jsonl.zst");
    let encoded = zstd::stream::encode_all(br#"{"text":"compressed zstd"}"#.as_slice(), 0)
        .expect("zstd fixture should encode");
    fs::write(&path, encoded).expect("fixture should write");

    let mut dataset = JsonlDataset::open(&path).expect("zstd dataset should open");
    let sample = dataset
        .next_sample()
        .expect("zstd row should read")
        .expect("sample should exist");

    assert_eq!(dataset.compression_stats().format, CompressionFormat::Zstd);
    assert_eq!(sample.payload.as_ref(), b"compressed zstd");

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn jsonl_dataset_streams_lz4_input() {
    let path = fixture_path("lz4", ".jsonl.lz4");
    let mut encoder = FrameEncoder::new(Vec::new());
    encoder
        .write_all(br#"{"text":"compressed lz4"}"#)
        .expect("lz4 fixture should write");
    let encoded = encoder.finish().expect("lz4 fixture should finish");
    fs::write(&path, encoded).expect("fixture should write");

    let mut dataset = JsonlDataset::open(&path).expect("lz4 dataset should open");
    let sample = dataset
        .next_sample()
        .expect("lz4 row should read")
        .expect("sample should exist");

    assert_eq!(dataset.compression_stats().format, CompressionFormat::Lz4);
    assert_eq!(sample.payload.as_ref(), b"compressed lz4");

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn mmap_reader_rejects_compressed_inputs() {
    let path = fixture_path("mmap-zstd", ".jsonl.zst");
    fs::write(&path, b"not checked").expect("fixture should write");
    let config = JsonlDatasetConfig::new(&path);

    assert!(matches!(
        MmapJsonlDataset::open_config(config),
        Err(DatasetError::UnsupportedCompression(CompressionKind::Auto))
    ));

    fs::remove_file(path).expect("fixture should remove");
}
