use std::time::Duration;

/// Pool configuration
#[derive(Clone, Debug)]
#[cfg_attr(feature = "config", derive(serde::Deserialize))]
pub struct PoolConfig {
    /// Maximum size of the pool
    pub max_size: usize,
    /// Timeouts
    #[cfg_attr(feature = "config", serde(default))]
    pub timeouts: Timeouts,
}

impl PoolConfig {
    /// Create pool config without any timeouts
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            timeouts: Timeouts::default(),
        }
    }
    /// Create pool config by reading it from the environment.
    /// ## Example environment
    /// ```env
    /// POOL_MAX_SIZE = 1s
    /// POOL_TIMEOUTS_WAIT = 1s
    /// POOL_TIMEOUTS_CREATE = 1s
    /// POOL_TIMEOUTS_RECYCLE = 1s
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
            timeouts: Timeouts::default(),
        }
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
