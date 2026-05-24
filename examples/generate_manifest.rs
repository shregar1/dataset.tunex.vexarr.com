use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use tunex_dataset::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = fixture_path("manifest", ".jsonl");
    fs::write(&path, "{\"id\":\"1\",\"text\":\"manifest sample\"}\n")?;

    let config = DatasetConfig::new(&path);
    let manifest = ManifestBuilder::from_jsonl_config("example-dataset", "0.1.0", &config)?
        .with_sample_count(1)
        .build();

    println!(
        "name={} version={} fingerprint={}",
        manifest.identity.name,
        manifest.identity.version,
        manifest.fingerprint.as_str()
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
