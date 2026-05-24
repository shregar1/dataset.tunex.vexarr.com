use bytes::Bytes;
use tunex_core::types::RawSample;
use tunex_dataset::{BatchBuilder, BatchSampler, PrefetchBatches};

fn sample(id: &str, payload: &'static [u8]) -> RawSample {
    RawSample::with_id(id, Bytes::from_static(payload))
}

#[test]
fn batch_builder_reuses_capacity_and_records_metadata() {
    let mut builder = BatchBuilder::with_capacity(4);
    builder.push(sample("a", b"hello"));
    builder.push(sample("b", b"world"));

    let batch = builder.finish().expect("batch should build");

    assert_eq!(batch.len(), 2);
    assert_eq!(batch.bytes, 10);
    assert_eq!(batch.metadata.first_sample_id.as_deref(), Some("a"));
    assert_eq!(batch.metadata.last_sample_id.as_deref(), Some("b"));
    assert!(builder.is_empty());
    assert_eq!(builder.stats().batches, 1);
}

#[test]
fn fixed_sampler_prefetches_batches() {
    let samples = vec![
        sample("a", b"one"),
        sample("b", b"two"),
        sample("c", b"three"),
    ];
    let mut prefetch = PrefetchBatches::spawn(samples, BatchSampler::fixed(2), 2);

    let first = prefetch
        .next_batch()
        .expect("first prefetch read should succeed")
        .expect("first batch should exist");
    let second = prefetch
        .next_batch()
        .expect("second prefetch read should succeed")
        .expect("second batch should exist");

    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 1);
    assert!(prefetch
        .next_batch()
        .expect("prefetch should close")
        .is_none());
    assert_eq!(prefetch.join().expect("worker should join").batches, 2);
}

#[test]
fn dynamic_sampler_splits_on_byte_threshold() {
    let samples = vec![
        sample("a", b"1111"),
        sample("b", b"2222"),
        sample("c", b"3333"),
    ];
    let prefetch = PrefetchBatches::spawn(samples, BatchSampler::dynamic(8, 8), 2);

    let first = prefetch
        .next_batch()
        .expect("first read should succeed")
        .expect("first batch should exist");
    let second = prefetch
        .next_batch()
        .expect("second read should succeed")
        .expect("second batch should exist");

    assert_eq!(first.len(), 2);
    assert_eq!(first.bytes, 8);
    assert_eq!(second.len(), 1);
    assert_eq!(second.bytes, 4);
}
