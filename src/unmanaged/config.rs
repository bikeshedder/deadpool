/// Pool configuration
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct PoolConfig {
    /// Maximum size of the pool
    pub max_size: usize,
}

impl PoolConfig {
    /// Create pool config without any timeouts
    pub fn new(max_size: usize) -> Self {
        Self { max_size }
    }
}

impl Default for PoolConfig {
    /// Create pool with default config. The `max_size` is set to
    /// `cpu_count * 4` ignoring any logical CPUs (Hyper-Threading).
    fn default() -> Self {
        Self {
            max_size: num_cpus::get_physical() * 4,
        }
    }
}
