use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::{DatasetError, JsonlDatasetConfig, MmapJsonlDataset};

fn fixture_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("tunex-dataset-mmap-{name}-{nanos}.jsonl"))
}

#[test]
fn mmap_jsonl_dataset_indexes_and_reads_samples() {
    let path = fixture_path("valid");
    fs::write(
        &path,
        "\n{\"id\":\"a\",\"text\":\"hello\"}\n{\"id\":\"b\",\"text\":\"world\"}\n",
    )
    .expect("fixture should write");

    let dataset = MmapJsonlDataset::open(&path).expect("mmap dataset should open");
    let first = dataset.get(0).expect("first sample should parse");
    let second = dataset.get(1).expect("second sample should parse");

    assert_eq!(dataset.len(), 2);
    assert_eq!(first.id.as_deref(), Some("a"));
    assert_eq!(first.payload.as_ref(), b"hello");
    assert_eq!(second.id.as_deref(), Some("b"));
    assert_eq!(second.payload.as_ref(), b"world");

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn mmap_jsonl_dataset_returns_borrowed_line_and_text_slices() {
    let path = fixture_path("slice");
    fs::write(&path, "{\"text\":\"hello\"}\n").expect("fixture should write");

    let dataset = MmapJsonlDataset::open(&path).expect("mmap dataset should open");
    let line = dataset.line(0).expect("line should exist");
    let record = dataset.record(0).expect("record should parse");
    let line_start = line.bytes.as_ptr() as usize;
    let line_end = line_start + line.bytes.len();
    let text_start = record.text.as_ptr() as usize;
    let text_end = text_start + record.text.len();

    assert_eq!(line.as_bytes(), br#"{"text":"hello"}"#);
    assert!(text_start >= line_start);
    assert!(text_end <= line_end);

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn mmap_jsonl_dataset_honors_max_samples() {
    let path = fixture_path("limit");
    fs::write(&path, "{\"text\":\"one\"}\n{\"text\":\"two\"}\n").expect("fixture should write");
    let mut config = JsonlDatasetConfig::new(&path);
    config.max_samples = Some(1);

    let dataset = MmapJsonlDataset::open_config(config).expect("mmap dataset should open");

    assert_eq!(dataset.len(), 1);
    assert!(dataset.get(0).is_ok());
    assert!(matches!(
        dataset.get(1),
        Err(DatasetError::IndexOutOfBounds { index: 1, len: 1 })
    ));

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn mmap_jsonl_dataset_reports_malformed_json_with_source_line() {
    let path = fixture_path("malformed");
    fs::write(&path, "\n{\"text\":\"ok\"}\n{bad json}\n").expect("fixture should write");

    let dataset = MmapJsonlDataset::open(&path).expect("mmap dataset should open");
    let error = dataset.record(1).expect_err("malformed row should fail");

    assert!(matches!(error, DatasetError::InvalidJson { line: 3, .. }));

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn mmap_jsonl_dataset_falls_back_to_owned_text_for_escaped_strings() {
    let path = fixture_path("escaped");
    fs::write(&path, "{\"text\":\"hello\\nworld\"}\n").expect("fixture should write");

    let dataset = MmapJsonlDataset::open(&path).expect("mmap dataset should open");
    let line = dataset.line(0).expect("line should exist");
    let record = dataset.record(0).expect("record should parse");
    let line_start = line.bytes.as_ptr() as usize;
    let line_end = line_start + line.bytes.len();
    let text_start = record.text.as_ptr() as usize;

    assert_eq!(record.text.as_ref(), b"hello\nworld");
    assert!(!(line_start..line_end).contains(&text_start));

    fs::remove_file(path).expect("fixture should remove");
}
