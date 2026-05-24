//! Stable dataset fingerprinting.

/// Stable dataset fingerprint represented as lowercase hex.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DatasetFingerprint(String);

impl DatasetFingerprint {
    /// Creates a fingerprint from a hex string.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the hex fingerprint.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Deterministic FNV-1a based fingerprint builder.
#[derive(Clone, Debug)]
pub struct FingerprintBuilder {
    hash: u64,
}

impl FingerprintBuilder {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    /// Creates an empty fingerprint builder.
    #[must_use]
    pub const fn new() -> Self {
        Self { hash: Self::OFFSET }
    }

    /// Adds bytes to the fingerprint.
    pub fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.hash ^= u64::from(*byte);
            self.hash = self.hash.wrapping_mul(Self::PRIME);
        }
    }

    /// Adds a string component with a separator.
    pub fn update_str(&mut self, value: &str) {
        self.update(value.as_bytes());
        self.update(&[0]);
    }

    /// Finalizes the fingerprint.
    #[must_use]
    pub fn finish(self) -> DatasetFingerprint {
        DatasetFingerprint(format!("{:016x}", self.hash))
    }
}

impl Default for FingerprintBuilder {
    fn default() -> Self {
        Self::new()
    }
}
