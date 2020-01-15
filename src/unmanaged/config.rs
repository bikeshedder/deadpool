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
    /// Create pool config by reading it from the environment.
    /// ## Example environment
    /// ```env
    /// POOL_MAX_SIZE = 1s
    /// ```
    /// ## Example usage
    /// ```rust,ignore
    /// Config::from_env("POOL")
    /// ```
    #[cfg(feature = "config")]
    pub fn from_env(prefix: &str) -> Result<PoolConfig, ::config_crate::ConfigError> {
        let mut cfg = ::config_crate::Config::new();
        cfg.merge(::config_crate::Environment::with_prefix(prefix))?;
        cfg.try_into()
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
