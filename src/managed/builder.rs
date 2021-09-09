use std::{fmt, marker::PhantomData, time::Duration};

use crate::Runtime;

use super::{
    hooks::{self, Hooks},
    Manager, Object, Pool, PoolConfig, Timeouts,
};

/// Possible errors returned when [`PoolBuilder::build()`] fails to build a
/// [`Pool`].
#[derive(Debug)]
pub enum BuildError<E> {
    /// Something is wrong with the configuration.
    /// See message string for details.
    Config(String),

    /// Backend reported an error when creating a [`Pool`].
    Backend(E),

    /// [`Runtime`] is required.
    NoRuntimeSpecified(String),
}

impl<E> From<E> for BuildError<E> {
    fn from(e: E) -> Self {
        Self::Backend(e)
    }
}

impl<E: std::fmt::Display> fmt::Display for BuildError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => {
                write!(f, "Error occurred while building the pool: Config: {}", msg)
            }
            Self::Backend(e) => write!(f, "Error occurred while building the pool: Backend: {}", e),
            Self::NoRuntimeSpecified(msg) => write!(
                f,
                "Error occurred while building the pool: NoRuntimeSpecified: {}",
                msg
            ),
        }
    }
}

impl<E: std::error::Error> std::error::Error for BuildError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Config(_) | Self::NoRuntimeSpecified(_) => None,
            Self::Backend(e) => Some(e),
        }
    }
}

/// Builder for [`Pool`]s.
///
/// Instances of this are created by calling the [`Pool::builder()`] method.
#[must_use = "builder does nothing itself, use `.build()` to build it"]
pub struct PoolBuilder<M, W = Object<M>>
where
    M: Manager,
    W: From<Object<M>>,
{
    pub(crate) manager: M,
    pub(crate) config: PoolConfig,
    pub(crate) runtime: Option<Runtime>,
    pub(crate) hooks: Hooks<M>,
    _wrapper: PhantomData<fn() -> W>,
}

impl<M, W> PoolBuilder<M, W>
where
    M: Manager,
    W: From<Object<M>>,
{
    pub(crate) fn new(manager: M) -> Self {
        Self {
            manager,
            config: PoolConfig::default(),
            runtime: None,
            hooks: Hooks::default(),
            _wrapper: PhantomData::default(),
        }
    }

    /// Builds the [`Pool`].
    ///
    /// # Errors
    ///
    /// See [`BuildError`] for details.
    pub fn build(self) -> Result<Pool<M, W>, BuildError<M::Error>> {
        // Return an error if a timeout is configured without runtime.
        let t = &self.config.timeouts;
        if (t.wait.is_some() || t.create.is_some() || t.recycle.is_some()) && self.runtime.is_none()
        {
            return Err(BuildError::NoRuntimeSpecified(
                "Timeouts require a runtime".to_string(),
            ));
        }
        Ok(Pool::from_builder(self))
    }

    /// Sets a [`PoolConfig`] to build the [`Pool`] with.
    pub fn config(mut self, value: PoolConfig) -> Self {
        self.config = value;
        self
    }

    /// Sets the [`PoolConfig::max_size`].
    pub fn max_size(mut self, value: usize) -> Self {
        self.config.max_size = value;
        self
    }

    /// Sets the [`PoolConfig::timeouts`].
    pub fn timeouts(mut self, value: Timeouts) -> Self {
        self.config.timeouts = value;
        self
    }

    /// Sets the [`Timeouts::wait`] value of the [`PoolConfig::timeouts`].
    pub fn wait_timeout(mut self, value: Option<Duration>) -> Self {
        self.config.timeouts.wait = value;
        self
    }

    /// Sets the [`Timeouts::create`] value of the [`PoolConfig::timeouts`].
    pub fn create_timeout(mut self, value: Option<Duration>) -> Self {
        self.config.timeouts.create = value;
        self
    }

    /// Sets the [`Timeouts::recycle`] value of the [`PoolConfig::timeouts`].
    pub fn recycle_timeout(mut self, value: Option<Duration>) -> Self {
        self.config.timeouts.recycle = value;
        self
    }

    /// Attaches a `post_create` hook.
    ///
    /// The given `hook` will be called each time right after a new [`Object`]
    /// has been created.
    pub fn post_create(mut self, hook: impl hooks::PostCreate<M> + 'static) -> Self {
        self.hooks.post_create.push(Box::new(hook));
        self
    }

    /// Attaches a `post_recycle` hook.
    ///
    /// The given `hook` will be called each time right after an [`Object`] has
    /// been recycled.
    pub fn post_recycle(mut self, hook: impl hooks::PostRecycle<M> + 'static) -> Self {
        self.hooks.post_recycle.push(Box::new(hook));
        self
    }

    /// Sets the [`Runtime`].
    ///
    /// # Important
    ///
    /// The [`Runtime`] is optional. Most [`Pool`]s don't need a
    /// [`Runtime`]. If want to utilize timeouts, however a [`Runtime`] must be
    /// specified as you will otherwise get a [`PoolError::NoRuntime`] when
    /// trying to use [`Pool::timeout_get()`].
    ///
    /// [`PoolBuilder::build()`] will fail with a
    /// [`BuildError::NoRuntimeSpecified`] if you try to build a
    /// [`Pool`] with timeouts and no [`Runtime`] specified.
    pub fn runtime(mut self, value: Runtime) -> Self {
        self.runtime = Some(value);
        self
    }
}
