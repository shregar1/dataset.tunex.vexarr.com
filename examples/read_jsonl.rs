use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = fixture_path("read-jsonl", ".jsonl");
    fs::write(&path, "{\"id\":\"1\",\"text\":\"hello TuneX\"}\n")?;

    let mut config = DatasetConfig::new(&path);
    config.pipeline.batch_size = 1;
    let mut dataset = JsonlDataset::from_config(config)?;
    let sample = dataset.next_sample()?.expect("sample");
    let batch = dataset.next_batch()?;

    println!(
        "sample={} batch_present={}",
        String::from_utf8_lossy(&sample.payload),
        batch.is_some()
    );

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
