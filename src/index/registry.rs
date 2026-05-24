//! In-memory dataset manifest registry.

use std::collections::BTreeMap;

use super::{DatasetFingerprint, DatasetManifest};

/// Registry of manifests keyed by fingerprint.
#[derive(Clone, Debug, Default)]
pub struct DatasetManifestRegistry {
    manifests: BTreeMap<DatasetFingerprint, DatasetManifest>,
}

impl DatasetManifestRegistry {
    /// Registers or replaces a manifest.
    pub fn register(&mut self, manifest: DatasetManifest) {
        self.manifests
            .insert(manifest.fingerprint.clone(), manifest);
    }

    /// Looks up a manifest by fingerprint.
    #[must_use]
    pub fn get(&self, fingerprint: &DatasetFingerprint) -> Option<&DatasetManifest> {
        self.manifests.get(fingerprint)
    }

    /// Returns all registered manifests.
    #[must_use]
    pub fn list(&self) -> Vec<&DatasetManifest> {
        self.manifests.values().collect()
    }

    /// Removes a manifest.
    pub fn remove(&mut self, fingerprint: &DatasetFingerprint) -> Option<DatasetManifest> {
        self.manifests.remove(fingerprint)
    }
}
