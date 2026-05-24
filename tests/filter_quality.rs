use bytes::Bytes;
use tunex_core::types::RawSample;
use tunex_dataset::{DatasetFilterConfig, FilterReason, JsonlDatasetConfig, ValidateDatasetConfig};

#[test]
fn length_and_empty_filters_reject_bad_samples() {
    let config = DatasetFilterConfig {
        min_chars: Some(3),
        max_chars: Some(8),
        remove_empty: true,
        ..DatasetFilterConfig::default()
    };
    let mut engine = config.build_engine().expect("engine");

    let samples = vec![
        sample(""),
        sample("ok"),
        sample("valid"),
        sample("too long text"),
    ];
    let accepted = engine.filter_samples(samples).expect("filter");

    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].payload.as_ref(), b"valid");
    assert_eq!(engine.stats().accepted, 1);
    assert_eq!(engine.stats().rejected, 3);
    assert_eq!(
        engine.stats().rejected_by_reason.get("empty_text"),
        Some(&1)
    );
    assert_eq!(engine.stats().rejected_by_reason.get("too_short"), Some(&1));
    assert_eq!(engine.stats().rejected_by_reason.get("too_long"), Some(&1));
}

#[test]
fn dedupe_filter_rejects_duplicate_text_deterministically() {
    let config = DatasetFilterConfig {
        dedupe: true,
        ..DatasetFilterConfig::default()
    };
    let mut engine = config.build_engine().expect("engine");

    let accepted = engine
        .filter_samples(vec![sample("same"), sample("same"), sample("other")])
        .expect("filter");

    assert_eq!(accepted.len(), 2);
    assert_eq!(
        engine.stats().rejected_by_reason.get("duplicate_text"),
        Some(&1)
    );
}

#[test]
fn regex_filter_rejects_matching_text() {
    let config = DatasetFilterConfig {
        reject_patterns: vec!["(?i)forbidden".to_owned()],
        ..DatasetFilterConfig::default()
    };
    let mut engine = config.build_engine().expect("engine");

    let accepted = engine
        .filter_samples(vec![
            sample("clean text"),
            sample("contains Forbidden phrase"),
        ])
        .expect("filter");

    assert_eq!(accepted.len(), 1);
    assert_eq!(accepted[0].payload.as_ref(), b"clean text");
    assert_eq!(
        engine.stats().rejected_by_reason.get("regex_matched"),
        Some(&1)
    );
}

#[test]
fn strict_mode_returns_error_on_rejection() {
    let config = DatasetFilterConfig {
        min_chars: Some(10),
        strict: true,
        ..DatasetFilterConfig::default()
    };
    let mut engine = config.build_engine().expect("engine");

    let error = engine.apply(sample("short")).expect_err("strict error");

    assert!(error.to_string().contains("strict filter rejected sample"));
}

#[test]
fn config_validation_rejects_invalid_filter_bounds_and_regex() {
    let mut config = JsonlDatasetConfig::default();
    config.filters.min_chars = Some(10);
    config.filters.max_chars = Some(5);
    config.filters.reject_patterns = vec!["[".to_owned()];

    let report = config.validate();

    assert!(report
        .errors
        .iter()
        .any(|error| error.path == "filters.min_chars"));
    assert!(report
        .errors
        .iter()
        .any(|error| error.path == "filters.reject_patterns"));
}

#[test]
fn filter_reasons_have_stable_labels() {
    assert_eq!(FilterReason::EmptyText.label(), "empty_text");
    assert_eq!(FilterReason::DuplicateText.label(), "duplicate_text");
}

fn sample(text: &'static str) -> RawSample {
    RawSample::new(Bytes::from_static(text.as_bytes()))
}
