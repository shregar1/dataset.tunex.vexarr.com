//! TuneX plugin contract for the dataset loader.

use tunex_core::plugin::{
    PluginCapability, PluginId, PluginKind, PluginMetadata, PluginName, PluginStatus,
    PluginVersion, TunexPlugin,
};

/// Stable TuneX dataset plugin name.
pub const DATASET_PLUGIN_NAME: &str = "tunex-dataset-jsonl";

/// Stable TuneX dataset plugin version.
pub const DATASET_PLUGIN_VERSION: PluginVersion = PluginVersion::new(0, 1, 0);

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
                PluginCapability::new("jsonl"),
                PluginCapability::new("streaming"),
                PluginCapability::new("mmap"),
                PluginCapability::new("simd-json"),
                PluginCapability::new("zstd"),
                PluginCapability::new("lz4"),
                PluginCapability::new("parallel-read"),
                PluginCapability::new("bounded-queues"),
                PluginCapability::new("estimated-throughput:750-mib-s"),
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
            PluginId::new(DATASET_PLUGIN_NAME),
            PluginName::new(DATASET_PLUGIN_NAME),
            DATASET_PLUGIN_VERSION,
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
