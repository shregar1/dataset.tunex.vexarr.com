use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::{DatasetError, JsonlDataset, JsonlDatasetConfig};

fn fixture_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("tunex-dataset-{name}-{nanos}.jsonl"))
}

#[test]
fn jsonl_dataset_reads_text_samples_and_skips_empty_lines() {
    let path = fixture_path("valid");
    fs::write(
        &path,
        "\n{\"id\":\"a\",\"text\":\"hello\"}\r\n{\"id\":\"b\",\"text\":\"world\"}\n",
    )
    .expect("fixture should write");

    let mut dataset = JsonlDataset::open(&path).expect("dataset should open");
    let first = dataset
        .next_sample()
        .expect("first read should succeed")
        .expect("first sample should exist");
    let second = dataset
        .next_sample()
        .expect("second read should succeed")
        .expect("second sample should exist");

    assert_eq!(first.id.as_deref(), Some("a"));
    assert_eq!(first.payload.as_ref(), b"hello");
    assert_eq!(second.id.as_deref(), Some("b"));
    assert_eq!(second.payload.as_ref(), b"world");
    assert!(dataset
        .next_sample()
        .expect("end of file should succeed")
        .is_none());

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn jsonl_dataset_honors_max_samples() {
    let path = fixture_path("limit");
    fs::write(
        &path,
        "{\"text\":\"one\"}\n{\"text\":\"two\"}\n{\"text\":\"three\"}\n",
    )
    .expect("fixture should write");
    let mut config = JsonlDatasetConfig::new(&path);
    config.max_samples = Some(2);

    let mut dataset = JsonlDataset::open_config(config).expect("dataset should open");

    assert!(dataset
        .next_sample()
        .expect("first read should succeed")
        .is_some());
    assert!(dataset
        .next_sample()
        .expect("second read should succeed")
        .is_some());
    assert!(dataset
        .next_sample()
        .expect("limit should stop iteration")
        .is_none());

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn jsonl_dataset_reports_malformed_json_line_number() {
    let path = fixture_path("malformed");
    fs::write(&path, "\n{\"text\":\"ok\"}\n{bad json}\n").expect("fixture should write");

    let mut dataset = JsonlDataset::open(&path).expect("dataset should open");
    assert!(dataset
        .next_sample()
        .expect("first valid row should read")
        .is_some());
    let error = dataset
        .next_sample()
        .expect_err("malformed row should fail");

    assert!(matches!(error, DatasetError::InvalidJson { line: 3, .. }));

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn jsonl_dataset_reports_missing_text_field() {
    let path = fixture_path("missing-text");
    fs::write(&path, "{\"body\":\"not text\"}\n").expect("fixture should write");

    let mut dataset = JsonlDataset::open(&path).expect("dataset should open");
    let error = dataset.next_sample().expect_err("missing text should fail");

    assert!(matches!(
        error,
        DatasetError::MissingText { line: 1, field } if field == "text"
    ));

    fs::remove_file(path).expect("fixture should remove");
}
