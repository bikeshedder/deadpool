use std::time::Duration;

use crate::Runtime;

/// Pool configuration
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct PoolConfig {
    /// Maximum size of the pool
    pub max_size: usize,
    /// Timeout for `Pool::get`
    pub timeout: Option<Duration>,
    /// Runtime
    #[serde(skip)]
    pub runtime: Runtime,
}

impl PoolConfig {
    /// Create pool config without any timeouts
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            timeout: None,
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
