use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = fixture_path("read-mmap", ".jsonl");
    fs::write(
        &path,
        "{\"id\":\"1\",\"text\":\"first\"}\n{\"id\":\"2\",\"text\":\"second\"}\n",
    )?;

    let dataset = MmapJsonlDataset::open(&path)?;
    let sample = dataset.get(1)?;

    println!(
        "rows={} second={}",
        dataset.len(),
        String::from_utf8_lossy(&sample.payload)
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
