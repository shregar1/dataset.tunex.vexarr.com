//! Bounded queue wrappers and stats.

use std::time::{Duration, Instant};

use crossbeam_channel::{bounded, Receiver, RecvError, SendTimeoutError, Sender};

/// Queue statistics snapshot.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct QueueStats {
    /// Queue capacity.
    pub capacity: usize,
    /// Total sends.
    pub sent: u64,
    /// Total receives.
    pub received: u64,
    /// Total time producers spent stalled.
    pub stall_time: Duration,
}

/// Bounded queue used by the pipeline runtime.
pub struct BoundedQueue<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
    capacity: usize,
}

impl<T> BoundedQueue<T> {
    /// Creates a bounded queue.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = bounded(capacity);
        Self {
            sender,
            receiver,
            capacity,
        }
    }

    /// Returns a sender clone.
    #[must_use]
    pub fn sender(&self) -> Sender<T> {
        self.sender.clone()
    }

    /// Returns a receiver clone.
    #[must_use]
    pub fn receiver(&self) -> Receiver<T> {
        self.receiver.clone()
    }

    /// Returns queue capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Sends an item and measures producer stall time.
pub fn send_with_backpressure<T>(
    sender: &Sender<T>,
    item: T,
    timeout: Duration,
) -> Result<Duration, SendTimeoutError<T>> {
    let started = Instant::now();
    sender.send_timeout(item, timeout)?;
    Ok(started.elapsed())
}

/// Receives one item.
pub fn recv<T>(receiver: &Receiver<T>) -> Result<T, RecvError> {
    receiver.recv()
}
