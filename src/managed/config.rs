use std::time::Duration;

use crate::Runtime;

/// Pool configuration
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct PoolConfig {
    /// Maximum size of the pool
    pub max_size: usize,
    /// Timeouts
    #[cfg_attr(feature = "config", serde(default))]
    pub timeouts: Timeouts,
    /// Runtime
    #[cfg_attr(feature = "config", serde(skip))]
    pub runtime: Runtime,
}

impl PoolConfig {
    /// Create pool config without any timeouts
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            timeouts: Timeouts::default(),
            runtime: Runtime::default(),
        }
    }
}

impl Default for PoolConfig {
    /// Create pool with default config. The `max_size` is set to
    /// `cpu_count * 4` ignoring any logical CPUs (Hyper-Threading).
    fn default() -> Self {
        Self::new(num_cpus::get_physical() * 4)
    }
}

/// Timeouts when getting objects from the pool
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
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
        Self::default()
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

impl Default for Timeouts {
    /// Create a timeout config with no timeouts set
    fn default() -> Self {
        Self {
            create: None,
            wait: None,
            recycle: None,
        }
    }
}
