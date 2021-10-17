use std::time::{Duration, Instant};

/// Statistics regarding an object returned by the pool
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Metrics {
    /// The instant when this object was created
    pub created: Instant,
    /// The instant when this object was last used
    pub recycled: Option<Instant>,
    /// The number of times the objects was recycled
    pub recycle_count: usize,
}

impl Metrics {
    /// Access the age of this object
    pub fn age(&self) -> Duration {
        self.created.elapsed()
    }
    /// Get the time elapsed when this object was last used
    pub fn last_used(&self) -> Duration {
        self.recycled.unwrap_or(self.created).elapsed()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            created: Instant::now(),
            recycled: None,
            recycle_count: 0,
        }
    }
}
