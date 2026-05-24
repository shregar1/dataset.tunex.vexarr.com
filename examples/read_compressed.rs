use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = fixture_path("read-compressed", ".jsonl.zst");
    let encoded = zstd::stream::encode_all(
        "{\"id\":\"1\",\"text\":\"compressed sample\"}\n".as_bytes(),
        0,
    )?;
    fs::write(&path, encoded)?;

    let mut dataset = JsonlDataset::open(&path)?;
    let sample = dataset.next_sample()?.expect("sample");

    println!(
        "format={:?} text={}",
        dataset.compression_stats().format,
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
