//! TuneX plugin contract for the dataset loader.

use tunex_core::plugin::{
    PluginCapability, PluginId, PluginKind, PluginMetadata, PluginName, PluginStatus,
    PluginVersion, TunexPlugin,
};

/// Dataset plugin metadata provider.
#[derive(Clone, Debug)]
pub struct DatasetPlugin {
    capabilities: Vec<PluginCapability>,
}

impl DatasetPlugin {
    /// Creates the default dataset plugin descriptor.
    #[must_use]
    pub fn new() -> Self {
        Self {
            capabilities: vec![
                PluginCapability::new("dataset:jsonl"),
                PluginCapability::new("dataset:stream"),
                PluginCapability::new("dataset:mmap"),
                PluginCapability::new("compression:zstd"),
                PluginCapability::new("compression:lz4"),
            ],
        }
    }
}

impl Default for DatasetPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TunexPlugin for DatasetPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata::new(
            PluginId::new("tunex-dataset-jsonl"),
            PluginName::new("TuneX JSONL Dataset"),
            PluginVersion::new(0, 1, 0),
            PluginKind::Dataset,
            self.capabilities.clone(),
        )
    }

    fn capabilities(&self) -> &[PluginCapability] {
        &self.capabilities
    }

    fn health(&self) -> PluginStatus {
        PluginStatus::Active
    }
}
