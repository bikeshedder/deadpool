use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    time::Duration,
};

use crate::Runtime;

use super::{
    hooks::{self, Hooks},
    Manager, Object, Pool, PoolConfig, Timeouts,
};

/// This error is returned when [`Pool::build()`] fails
/// to build the pool.
#[derive(Debug)]
pub enum BuildError<E> {
    /// Something is wrong with the configuration. See message string for details.
    Config(String),
    /// The backend reported an error when creating the pool
    Backend(E),
    /// A runtime is required
    NoRuntimeSpecified(String),
}

impl<E> fmt::Display for BuildError<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(msg) => write!(
                f,
                "An error occured while building the pool: Config: {}",
                msg
            ),
            Self::Backend(e) => write!(
                f,
                "An error occured while building the pool: Backend: {}",
                e
            ),
            Self::NoRuntimeSpecified(msg) => write!(
                f,
                "An error occured while building the pool: NoRuntimeSpecified: {}",
                msg
            ),
        }
    }
}

impl<E> std::error::Error for BuildError<E> where E: std::error::Error {}

impl<E> From<E> for BuildError<E> {
    fn from(e: E) -> Self {
        Self::Backend(e)
    }
}

/// Builder for pools
///
/// Instances of this are created by calling the [`Pool::builder()`]
/// method.
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
    /// Build the pool object
    pub fn build(self) -> Result<Pool<M, W>, BuildError<M::Error>> {
        // Return an error if a timeout is configured without a runtime
        let t = &self.config.timeouts;
        if (t.wait.is_some() || t.create.is_some() || t.recycle.is_some()) && self.runtime.is_none()
        {
            return Err(BuildError::NoRuntimeSpecified(
                "Timeouts require a runtime".to_string(),
            ));
        }
        Ok(Pool::from_builder(self))
    }
    /// Set the pool configuration
    pub fn config(mut self, value: PoolConfig) -> Self {
        self.config = value;
        self
    }
    /// Set the [PoolConfig::max_size]
    pub fn max_size(mut self, value: usize) -> Self {
        self.config.max_size = value;
        self
    }
    /// Set the [PoolConfig::timeouts]
    pub fn timeouts(mut self, value: Timeouts) -> Self {
        self.config.timeouts = value;
        self
    }
    /// Set the [Timeouts::wait] value of the [PoolConfig::timeouts]
    pub fn wait_timeout(mut self, value: Option<Duration>) -> Self {
        self.config.timeouts.wait = value;
        self
    }
    /// Set the [Timeouts::create] value of the [PoolConfig::timeouts]
    pub fn create_timeout(mut self, value: Option<Duration>) -> Self {
        self.config.timeouts.create = value;
        self
    }
    /// Set the [Timeouts::recycle] value of the [PoolConfig::timeouts]
    pub fn recycle_timeout(mut self, value: Option<Duration>) -> Self {
        self.config.timeouts.recycle = value;
        self
    }
    /// Attach a post_create hook. The given function will be called right
    /// after a object has been created.
    pub fn post_create(mut self, hook: impl hooks::PostCreate<M> + 'static) -> Self {
        self.hooks.post_create.push(Box::new(hook));
        self
    }
    /// Attach a post_recycle hook. The given function will be called right
    /// after a object has been recycled.
    pub fn post_recycle(mut self, hook: impl hooks::PostRecycle<M> + 'static) -> Self {
        self.hooks.post_recycle.push(Box::new(hook));
        self
    }
    /// Set the [Runtime]
    ///
    /// **Important**: The runtime is optional. Most pools do not need a
    /// runtime. If want to utilize timeouts however a runtime must be
    /// specified as you will otherwise get a [`PoolError::NoRuntime`]
    /// error when trying to use [`Pool::timeout_get`]. [`PoolBuilder::build`]
    /// will fail with [`BuildError::NoRuntimeSpecified`] if you try to build a
    /// pool with timeouts and no runtime specified.
    pub fn runtime(mut self, value: Runtime) -> Self {
        self.runtime = Some(value);
        self
    }
}
