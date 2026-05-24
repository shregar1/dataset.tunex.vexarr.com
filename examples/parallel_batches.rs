use std::{
    fs,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tunex_dataset::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = fixture_path("parallel", ".jsonl");
    let mut input = String::new();
    for index in 0..128 {
        input.push_str(&format!(
            "{{\"id\":\"{index}\",\"text\":\"sample {index}\"}}\n"
        ));
    }
    fs::write(&path, input)?;

    let mut config = DatasetConfig::new(&path);
    config.pipeline.batch_size = 32;
    let mut pipeline = ParallelJsonlPipeline::open(
        config,
        PipelineOptions {
            chunk_lines: 32,
            chunk_queue_capacity: 2,
            batch_queue_capacity: 2,
            send_timeout: Duration::from_secs(1),
        },
    )?;

    let mut batches = 0;
    while let Some(batch) = pipeline.next_batch()? {
        batches += 1;
        println!("batch samples={} bytes={}", batch.len(), batch.bytes);
    }
    let stats = pipeline.join()?;
    println!("batches={batches} queue_stall={:?}", stats.queue_stall_time);

    fs::remove_file(path)?;
    Ok(())
}

fn fixture_path(name: &str, suffix: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "tunex-example-{name}-{}{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        suffix
    ))
}
