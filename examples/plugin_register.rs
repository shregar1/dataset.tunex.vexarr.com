use std::sync::Arc;

use tunex_core::plugin::{PluginKind, PluginRegistry};
use tunex_dataset::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut registry = PluginRegistry::new();
    registry.register(Arc::new(TunexDatasetPlugin::new()))?;

    let datasets = registry.lookup_by_kind(PluginKind::Dataset);
    println!("dataset_plugins={}", datasets.len());

    Ok(())
}
