use std::{
    fs,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tunex_dataset::{ParallelJsonlPipeline, PipelineOptions};

fn fixture_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("tunex-pipeline-{name}-{nanos}.jsonl"))
}

#[test]
fn parallel_pipeline_emits_bounded_sample_batches() {
    let path = fixture_path("batches");
    let mut input = String::new();
    for index in 0..10 {
        input.push_str(&format!(
            "{{\"id\":\"sample-{index}\",\"text\":\"payload-{index}\"}}\n"
        ));
    }
    fs::write(&path, input).expect("fixture should write");

    let options = PipelineOptions {
        chunk_lines: 4,
        chunk_queue_capacity: 2,
        batch_queue_capacity: 2,
        send_timeout: Duration::from_secs(1),
    };
    let mut pipeline =
        ParallelJsonlPipeline::open(tunex_dataset::JsonlDatasetConfig::new(&path), options)
            .expect("pipeline should open");

    let mut samples = Vec::new();
    while let Some(batch) = pipeline.next_batch().expect("batch should read") {
        assert!(batch.len() <= 4);
        samples.extend(batch.samples);
    }
    let stats = pipeline.join().expect("pipeline should join");

    assert_eq!(samples.len(), 10);
    assert_eq!(samples[0].payload.as_ref(), b"payload-0");
    assert_eq!(stats.samples, 10);
    assert_eq!(stats.chunks, 3);

    fs::remove_file(path).expect("fixture should remove");
}

#[test]
fn parallel_pipeline_honors_max_samples() {
    let path = fixture_path("limit");
    fs::write(
        &path,
        "{\"text\":\"one\"}\n{\"text\":\"two\"}\n{\"text\":\"three\"}\n",
    )
    .expect("fixture should write");
    let mut config = tunex_dataset::JsonlDatasetConfig::new(&path);
    config.max_samples = Some(2);
    let pipeline = ParallelJsonlPipeline::open(
        config,
        PipelineOptions {
            chunk_lines: 8,
            ..PipelineOptions::default()
        },
    )
    .expect("pipeline should open");

    let batch = pipeline
        .next_batch()
        .expect("batch read should succeed")
        .expect("batch should exist");
    assert_eq!(batch.len(), 2);
    assert!(pipeline
        .next_batch()
        .expect("pipeline should close cleanly")
        .is_none());

    fs::remove_file(path).expect("fixture should remove");
}
