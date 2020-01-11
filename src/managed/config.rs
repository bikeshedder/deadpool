use std::time::Duration;

/// Pool configuration
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Maximum size of the pool
    pub max_size: usize,
    /// Timeouts
    pub timeouts: Timeouts,
}

impl PoolConfig {
    /// Create pool config without any timeouts
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            timeouts: Timeouts::new(),
        }
    }
}

/// Timeouts when getting objects from the pool
#[derive(Clone, Debug)]
pub struct Timeouts {
    /// Timeout when waiting for a slot to become available
    pub wait: Option<Duration>,
    /// Timeout when creating a new object
    pub create: Option<Duration>,
    /// Timeout when recycling an object
    pub recycle: Option<Duration>,
}

impl Timeouts {
    /// Create a timeout config with no timeouts set
    pub fn new() -> Self {
        Self {
            create: None,
            wait: None,
            recycle: None,
        }
    }
    /// Create a timeout config with no timeouts set
    pub fn wait_millis(wait: u64) -> Self {
        Self {
            create: None,
            wait: Some(Duration::from_millis(wait)),
            recycle: None,
        }
    }
}
