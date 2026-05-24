//! Batch prefetch pipeline.

use std::{
    thread::{self, JoinHandle},
    time::Instant,
};

use crossbeam_channel::{bounded, Receiver};
use tunex_core::types::RawSample;

use crate::{DatasetError, DatasetResult};

use super::{BatchBuilder, BatchSampler, BatchStats, DatasetBatch};

type BatchMessage = DatasetResult<DatasetBatch>;

/// Background prefetcher for dataset batches.
pub struct PrefetchBatches {
    receiver: Receiver<BatchMessage>,
    handle: Option<JoinHandle<BatchStats>>,
}

impl PrefetchBatches {
    /// Starts prefetching batches from an input iterator.
    #[must_use]
    pub fn spawn<I>(samples: I, sampler: BatchSampler, prefetch_batches: usize) -> Self
    where
        I: IntoIterator<Item = RawSample> + Send + 'static,
        I::IntoIter: Send,
    {
        let (sender, receiver) = bounded(prefetch_batches);
        let handle = thread::spawn(move || {
            let mut builder = BatchBuilder::with_capacity(sampler.capacity_hint());
            let mut stats = BatchStats::default();

            for sample in samples {
                if sampler.would_exceed(builder.bytes(), &sample) {
                    send_finished(&sender, &mut builder, &mut stats);
                }
                builder.push(sample);
                if sampler.should_emit(builder.len(), builder.bytes()) {
                    send_finished(&sender, &mut builder, &mut stats);
                }
            }
            send_finished(&sender, &mut builder, &mut stats);
            stats
        });

        Self {
            receiver,
            handle: Some(handle),
        }
    }

    /// Returns the next prefetched batch.
    ///
    /// # Errors
    ///
    /// Returns any batch construction error from the background worker.
    pub fn next_batch(&self) -> DatasetResult<Option<DatasetBatch>> {
        match self.receiver.recv() {
            Ok(Ok(batch)) => Ok(Some(batch)),
            Ok(Err(error)) => Err(error),
            Err(_) => Ok(None),
        }
    }

    /// Joins the prefetch worker.
    ///
    /// # Errors
    ///
    /// Returns an error if the worker panicked.
    pub fn join(&mut self) -> DatasetResult<BatchStats> {
        let Some(handle) = self.handle.take() else {
            return Ok(BatchStats::default());
        };
        handle
            .join()
            .map_err(|_| DatasetError::Pipeline("batch prefetch worker panicked".to_owned()))
    }
}

impl Drop for PrefetchBatches {
    fn drop(&mut self) {
        let _ = self.join();
    }
}

fn send_finished(
    sender: &crossbeam_channel::Sender<BatchMessage>,
    builder: &mut BatchBuilder,
    stats: &mut BatchStats,
) {
    let started = Instant::now();
    if let Some(batch) = builder.finish() {
        stats.record_batch(batch.len(), batch.bytes, started.elapsed());
        stats.queue_wait_time += started.elapsed();
        let _ = sender.send(Ok(batch));
    }
}
