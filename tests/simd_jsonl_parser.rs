use bytes::Bytes;
use tunex_dataset::{JsonlDatasetConfig, SimdJsonlParser};

#[test]
fn simd_parser_batch_parses_jsonl_records() {
    let parser = SimdJsonlParser::new(&JsonlDatasetConfig::default());
    let records = parser
        .parse_batch([
            (1, Bytes::from_static(br#"{"text":"one"}"#)),
            (2, Bytes::from_static(br#"{"text":"two"}"#)),
        ])
        .expect("batch should parse");

    assert_eq!(records.len(), 2);
    assert_eq!(records[0].text.as_ref(), b"one");
    assert_eq!(records[1].text.as_ref(), b"two");
}

#[test]
fn simd_parser_uses_fast_path_for_unescaped_text() {
    let parser = SimdJsonlParser::new(&JsonlDatasetConfig::default());
    let line = Bytes::from_static(br#"{"text":"borrowed"}"#);
    let line_start = line.as_ptr() as usize;
    let line_end = line_start + line.len();
    let record = parser.parse_bytes(1, line).expect("line should parse");
    let text_start = record.text.as_ptr() as usize;
    let text_end = text_start + record.text.len();

    assert_eq!(record.text.as_ref(), b"borrowed");
    assert!(text_start >= line_start);
    assert!(text_end <= line_end);
}
