use std::time::Duration;

/// Pool configuration
pub struct PoolConfig {
    /// Maximum size of the pool
    pub max_size: usize,
    /// Timeout when creating a new object
    pub create_timeout: Option<Duration>,
    /// Timeout when waiting for an object to become available or None
    pub wait_timeout: Option<Duration>,
    /// Timeout when recycling an object
    pub recycle_timeout: Option<Duration>,
}

impl PoolConfig {
    /// Create pool without any timeouts
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            create_timeout: None,
            wait_timeout: None,
            recycle_timeout: None,
        }
    }
}
