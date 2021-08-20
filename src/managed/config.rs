use std::time::Duration;

/// [`Pool`] configuration.
///
/// [`Pool`]: super::Pool
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct PoolConfig {
    /// Maximum size of the [`Pool`].
    ///
    /// [`Pool`]: super::Pool
    pub max_size: usize,

    /// Timeouts of the [`Pool`].
    ///
    /// [`Pool`]: super::Pool
    #[cfg_attr(feature = "serde", serde(default))]
    pub timeouts: Timeouts,
}

impl PoolConfig {
    /// Creates a new [`PoolConfig`] without any timeouts and with the provided
    /// `max_size`.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            timeouts: Timeouts::default(),
        }
    }
}

impl Default for PoolConfig {
    /// Creates a new [`PoolConfig`] with the `max_size` being set to
    /// `cpu_count * 4` ignoring any logical CPUs (Hyper-Threading).
    fn default() -> Self {
        Self::new(num_cpus::get_physical() * 4)
    }
}

/// Timeouts when getting [`Object`]s from a [`Pool`].
///
/// [`Object`]: super::Object
/// [`Pool`]: super::Pool
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Timeouts {
    /// Timeout when waiting for a slot to become available.
    pub wait: Option<Duration>,

    /// Timeout when creating a new object.
    pub create: Option<Duration>,

    /// Timeout when recycling an object.
    pub recycle: Option<Duration>,
}

impl Timeouts {
    /// Create an empty [`Timeouts`] config (no timeouts set).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new [`Timeouts`] config with only the `wait` timeout being
    /// set.
    #[must_use]
    pub fn wait_millis(wait: u64) -> Self {
        Self {
            create: None,
            wait: Some(Duration::from_millis(wait)),
            recycle: None,
        }
    }
}

// Implemented manually to provide a custom documentation.
impl Default for Timeouts {
    /// Creates an empty [`Timeouts`] config (no timeouts set).
    fn default() -> Self {
        Self {
            create: None,
            wait: None,
            recycle: None,
        }
    }
}
