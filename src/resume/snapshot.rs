//! Serializable dataset resume snapshots.

use super::{DatasetCursor, DatasetResumeState};
use crate::DatasetResult;

/// Versioned dataset resume snapshot stored alongside trainer checkpoints.
#[derive(Clone, Debug, PartialEq)]
pub struct DatasetResumeSnapshot {
    /// Snapshot format version.
    pub version: u32,
    /// Resume state payload.
    pub state: DatasetResumeState,
    /// Optional checkpoint id this snapshot belongs to.
    pub checkpoint_id: Option<String>,
    /// Optional trainer step this snapshot belongs to.
    pub checkpoint_step: Option<u64>,
}

impl DatasetResumeSnapshot {
    /// Current snapshot schema version.
    pub const VERSION: u32 = 1;

    /// Creates a snapshot from resume state.
    pub fn new(state: DatasetResumeState) -> DatasetResult<Self> {
        state.validate()?;
        Ok(Self {
            version: Self::VERSION,
            state,
            checkpoint_id: None,
            checkpoint_step: None,
        })
    }

    /// Creates a snapshot directly from a cursor.
    pub fn from_cursor(cursor: DatasetCursor) -> DatasetResult<Self> {
        Self::new(DatasetResumeState::new(cursor))
    }

    /// Attaches checkpoint identity.
    #[must_use]
    pub fn with_checkpoint(mut self, checkpoint_id: impl Into<String>, step: u64) -> Self {
        self.checkpoint_id = Some(checkpoint_id.into());
        self.checkpoint_step = Some(step);
        self
    }

    /// Returns the cursor embedded in this snapshot.
    #[must_use]
    pub const fn cursor(&self) -> &DatasetCursor {
        &self.state.cursor
    }
}
