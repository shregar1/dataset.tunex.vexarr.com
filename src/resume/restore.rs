//! Restore planning from dataset resume snapshots and checkpoint metadata.

use tunex_core::checkpoint::CheckpointMetadata;

use crate::{DatasetError, DatasetResult};

use super::DatasetResumeSnapshot;

/// Policy used when checkpoint metadata and dataset resume state differ.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RestorePolicy {
    /// Require checkpoint id and step to match when present.
    #[default]
    Strict,
    /// Allow dataset state to be restored even when checkpoint metadata is absent.
    BestEffort,
}

/// Restore plan derived from a checkpoint and dataset cursor snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckpointRestorePlan {
    /// Dataset resume snapshot.
    pub snapshot: DatasetResumeSnapshot,
    /// Trainer checkpoint step.
    pub checkpoint_step: Option<u64>,
    /// Restore policy.
    pub policy: RestorePolicy,
}

impl CheckpointRestorePlan {
    /// Builds a restore plan from a dataset snapshot and optional checkpoint metadata.
    pub fn from_snapshot(
        snapshot: DatasetResumeSnapshot,
        checkpoint: Option<&CheckpointMetadata>,
        policy: RestorePolicy,
    ) -> DatasetResult<Self> {
        if let Some(checkpoint) = checkpoint {
            if policy == RestorePolicy::Strict {
                if let Some(snapshot_id) = &snapshot.checkpoint_id {
                    if snapshot_id != checkpoint.id.as_str() {
                        return Err(DatasetError::InvalidConfig(format!(
                            "dataset resume checkpoint id `{snapshot_id}` does not match trainer checkpoint `{}`",
                            checkpoint.id
                        )));
                    }
                }
                if let Some(snapshot_step) = snapshot.checkpoint_step {
                    if snapshot_step != checkpoint.step {
                        return Err(DatasetError::InvalidConfig(format!(
                            "dataset resume checkpoint step {snapshot_step} does not match trainer checkpoint step {}",
                            checkpoint.step
                        )));
                    }
                }
            }
        } else if policy == RestorePolicy::Strict {
            return Err(DatasetError::InvalidConfig(
                "strict restore requires checkpoint metadata".to_owned(),
            ));
        }

        snapshot.state.validate()?;
        Ok(Self {
            snapshot,
            checkpoint_step: checkpoint.map(|metadata| metadata.step),
            policy,
        })
    }
}
