//! Managed version of the pool.
//!
//! "Managed" means that it requires a [`Manager`] which is responsible for
//! creating and recycling objects as they are needed.
//!
//! # Example
//!
//! ```rust
//! use async_trait::async_trait;
//! use deadpool::managed;
//!
//! #[derive(Debug)]
//! enum Error { Fail }
//!
//! struct Computer {}
//!
//! impl Computer {
//!     async fn get_answer(&self) -> i32 {
//!         42
//!     }
//! }
//!
//! struct Manager {}
//!
//! #[async_trait]
//! impl managed::Manager for Manager {
//!     type Type = Computer;
//!     type Error = Error;
//!
//!     async fn create(&self) -> Result<Computer, Error> {
//!         Ok(Computer {})
//!     }
//!     async fn recycle(&self, conn: &mut Computer) -> managed::RecycleResult<Error> {
//!         Ok(())
//!     }
//! }
//!
//! type Pool = managed::Pool<Manager>;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mgr = Manager {};
//!     let pool = Pool::builder(mgr).max_size(16).build().unwrap();
//!     let mut conn = pool.get().await.unwrap();
//!     let answer = conn.get_answer().await;
//!     assert_eq!(answer, 42);
//! }
//! ```
//!
//! For a more complete example please see
//! [`deadpool-postgres`](https://crates.io/crates/deadpool-postgres) crate.

mod builder;
mod config;
mod errors;
pub mod hooks;
pub mod sync;

use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicIsize, AtomicUsize, Ordering},
        Arc, Mutex, Weak,
    },
    time::Duration,
};

use async_trait::async_trait;
use tokio::sync::{Semaphore, TryAcquireError};

use crate::runtime::Runtime;

pub use crate::Status;

pub use self::{
    builder::{BuildError, PoolBuilder},
    config::{PoolConfig, Timeouts},
    errors::{PoolError, RecycleError, TimeoutType},
};

/// Result type of the [`Manager::recycle()`] method.
pub type RecycleResult<E> = Result<(), RecycleError<E>>;

/// Manager responsible for creating new [`Object`]s or recycling existing ones.
#[async_trait]
pub trait Manager: Sync + Send {
    /// Type of [`Object`]s that this [`Manager`] creates and recycles.
    type Type;
    /// Error that this [`Manager`] can return when creating and/or recycling
    /// [`Object`]s.
    type Error;

    /// Creates a new instance of [`Manager::Type`].
    async fn create(&self) -> Result<Self::Type, Self::Error>;

    /// Tries to recycle an instance of [`Manager::Type`].
    ///
    /// # Errors
    ///
    /// Returns [`Manager::Error`] if the instance couldn't be recycled.
    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error>;

    /// Detaches an instance of [`Manager::Type`] from this [`Manager`].
    ///
    /// This method is called when using the [`Object::take()`] method for
    /// removing an [`Object`] from a [`Pool`]. If the [`Manager`] doesn't hold
    /// any references to the handed out [`Object`]s then the default
    /// implementation can be used which does nothing.
    fn detach(&self, _obj: &mut Self::Type) {}
}

#[derive(Debug)]
enum ObjectState {
    Waiting,
    Receiving,
    Creating,
    Recycling,
    Ready,
    Taken,
    Dropped,
}

/// Wrapper around the actual pooled object which implements [`Deref`],
/// [`DerefMut`] and [`Drop`] traits.
///
/// Use this object just as if it was of type `T` and upon leaving a scope the
/// [`Drop::drop()`] will take care of returning it to the pool.
#[derive(Debug)]
#[must_use]
pub struct Object<M: Manager> {
    /// Actual pooled object.
    obj: Option<M::Type>,

    /// Current state of the object.
    state: ObjectState,

    /// Pool to return the pooled object to.
    pool: Weak<PoolInner<M>>,
}

impl<M: Manager> Object<M> {
    /// Takes this [`Object`] from its [`Pool`] permanently. This reduces the
    /// size of the [`Pool`].
    #[must_use]
    pub fn take(mut this: Self) -> M::Type {
        this.state = ObjectState::Taken;
        if let Some(pool) = this.pool.upgrade() {
            pool.manager.detach(&mut this);
        }
        this.obj.take().unwrap()
    }

    /// Returns the [`Pool`] this [`Object`] belongs to.
    ///
    /// Since [`Object`]s only hold a [`Weak`] reference to the [`Pool`] they
    /// come from, this can fail and return [`None`] instead.
    pub fn pool(this: &Self) -> Option<Pool<M>> {
        this.pool.upgrade().map(|inner| Pool {
            inner,
            _wrapper: PhantomData::default(),
        })
    }
}

impl<M: Manager> Drop for Object<M> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            match self.state {
                ObjectState::Waiting => {
                    let _ = pool.available.fetch_add(1, Ordering::Relaxed);
                }
                ObjectState::Receiving => {
                    let _ = pool.available.fetch_add(1, Ordering::Relaxed);
                    pool.semaphore.add_permits(1);
                }
                ObjectState::Creating | ObjectState::Taken => {
                    let _ = pool.size.fetch_sub(1, Ordering::Relaxed);
                    pool.semaphore.add_permits(1);
                }
                ObjectState::Recycling | ObjectState::Ready => {
                    let _ = pool.available.fetch_add(1, Ordering::Relaxed);
                    let obj = self.obj.take().unwrap();
                    {
                        let mut queue = pool.queue.lock().unwrap();
                        queue.push_back(obj);
                    }
                    pool.semaphore.add_permits(1);
                    // The pool might have been closed in the mean time.
                    // Hand over control to the `_cleanup` method which
                    // takes care of this.
                    pool.clean_up();
                }
                ObjectState::Dropped => {
                    // The object has already been dropped.
                }
            }
        }
        self.obj = None;
        self.state = ObjectState::Dropped;
    }
}

impl<M: Manager> Deref for Object<M> {
    type Target = M::Type;
    fn deref(&self) -> &M::Type {
        self.obj.as_ref().unwrap()
    }
}

impl<M: Manager> DerefMut for Object<M> {
    fn deref_mut(&mut self) -> &mut M::Type {
        self.obj.as_mut().unwrap()
    }
}

impl<M: Manager> AsRef<M::Type> for Object<M> {
    fn as_ref(&self) -> &M::Type {
        self
    }
}

impl<M: Manager> AsMut<M::Type> for Object<M> {
    fn as_mut(&mut self) -> &mut M::Type {
        self
    }
}

/// Generic object and connection pool.
///
/// This struct can be cloned and transferred across thread boundaries and uses
/// reference counting for its internal state.
pub struct Pool<M: Manager, W: From<Object<M>> = Object<M>> {
    inner: Arc<PoolInner<M>>,
    _wrapper: PhantomData<fn() -> W>,
}

// Implemented manually to avoid unnecessary trait bound on `W` type parameter.
impl<M, W> fmt::Debug for Pool<M, W>
where
    M: fmt::Debug + Manager,
    M::Type: fmt::Debug,
    W: From<Object<M>>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pool")
            .field("inner", &self.inner)
            .field("wrapper", &self._wrapper)
            .finish()
    }
}

impl<M: Manager, W: From<Object<M>>> Clone for Pool<M, W> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _wrapper: PhantomData::default(),
        }
    }
}

impl<M: Manager, W: From<Object<M>>> Pool<M, W> {
    /// Instantiates a builder for a new [`Pool`].
    ///
    /// This is the only way to create a [`Pool`] instance.
    pub fn builder(manager: M) -> PoolBuilder<M, W> {
        PoolBuilder::new(manager)
    }

    pub(crate) fn from_builder(builder: PoolBuilder<M, W>) -> Self {
        Self {
            inner: Arc::new(PoolInner {
                manager: Box::new(builder.manager),
                queue: Mutex::new(VecDeque::with_capacity(builder.config.max_size)),
                size: AtomicUsize::new(0),
                available: AtomicIsize::new(0),
                semaphore: Semaphore::new(builder.config.max_size),
                config: builder.config,
                hooks: builder.hooks,
                runtime: builder.runtime,
            }),
            _wrapper: PhantomData::default(),
        }
    }

    /// Retrieves an [`Object`] from this [`Pool`] or waits for the one to
    /// become available.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn get(&self) -> Result<W, PoolError<M::Error>> {
        self.timeout_get(&self.inner.config.timeouts).await
    }

    /// Retrieves an [`Object`] from this [`Pool`] and doesn't wait if there is
    /// currently no [`Object`] is available and the maximum [`Pool`] size has
    /// been reached.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn try_get(&self) -> Result<W, PoolError<M::Error>> {
        let mut timeouts = self.inner.config.timeouts.clone();
        timeouts.wait = Some(Duration::from_secs(0));
        self.timeout_get(&timeouts).await
    }

    /// Retrieves an [`Object`] from this [`Pool`] using a different `timeout`
    /// than the configured one.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn timeout_get(&self, timeouts: &Timeouts) -> Result<W, PoolError<M::Error>> {
        let _ = self.inner.available.fetch_sub(1, Ordering::Relaxed);

        let mut obj = Object {
            obj: None,
            state: ObjectState::Waiting,
            pool: Arc::downgrade(&self.inner),
        };

        let non_blocking = match timeouts.wait {
            Some(t) => t.as_nanos() == 0,
            None => false,
        };

        let permit = if non_blocking {
            self.inner.semaphore.try_acquire().map_err(|e| match e {
                TryAcquireError::Closed => PoolError::Closed,
                TryAcquireError::NoPermits => PoolError::Timeout(TimeoutType::Wait),
            })?
        } else {
            apply_timeout(
                &self.inner.runtime,
                TimeoutType::Wait,
                self.inner.config.timeouts.wait,
                async {
                    self.inner
                        .semaphore
                        .acquire()
                        .await
                        .map_err(|_| PoolError::Closed)
                },
            )
            .await?
        };

        permit.forget();

        loop {
            obj.state = ObjectState::Receiving;
            let inner_obj = {
                let mut queue = self.inner.queue.lock().unwrap();
                queue.pop_front()
            };
            match inner_obj {
                Some(inner_obj) => {
                    // Recycle existing object
                    obj.state = ObjectState::Recycling;
                    obj.obj = Some(inner_obj);
                    match apply_timeout(
                        &self.inner.runtime,
                        TimeoutType::Recycle,
                        self.inner.config.timeouts.recycle,
                        self.inner.manager.recycle(&mut obj),
                    )
                    .await
                    {
                        Ok(_) => {
                            // Apply post_recycle hooks
                            for hook in self.inner.hooks.post_recycle.iter() {
                                hook.post_recycle(&mut obj.obj.as_mut().unwrap())
                                    .await
                                    .map_err(PoolError::PostRecycleHook)?;
                            }
                            break;
                        }
                        Err(_) => {
                            let _ = self.inner.available.fetch_sub(1, Ordering::Relaxed);
                            let _ = self.inner.size.fetch_sub(1, Ordering::Relaxed);
                            continue;
                        }
                    }
                }
                None => {
                    // Create new object
                    obj.state = ObjectState::Creating;
                    let _ = self.inner.available.fetch_add(1, Ordering::Relaxed);
                    let _ = self.inner.size.fetch_add(1, Ordering::Relaxed);
                    obj.obj = Some(
                        apply_timeout(
                            &self.inner.runtime,
                            TimeoutType::Create,
                            self.inner.config.timeouts.create,
                            self.inner.manager.create(),
                        )
                        .await?,
                    );
                    // Apply post_create hooks
                    for hook in self.inner.hooks.post_create.iter() {
                        hook.post_create(&mut obj.obj.as_mut().unwrap())
                            .await
                            .map_err(PoolError::PostCreateHook)?;
                    }
                    break;
                }
            }
        }

        obj.state = ObjectState::Ready;
        Ok(obj.into())
    }

    /// Closes this [`Pool`].
    ///
    /// All current and future tasks waiting for [`Object`]s will return
    /// [`PoolError::Closed`] immediately.
    pub fn close(&self) {
        self.inner.semaphore.close();
        self.inner.clean_up();
    }

    /// Indicates whether this [`Pool`] has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    /// Retrieves [`Status`] of this [`Pool`].
    #[must_use]
    pub fn status(&self) -> Status {
        let max_size = self.inner.config.max_size;
        let size = self.inner.size.load(Ordering::Relaxed);
        let available = self.inner.available.load(Ordering::Relaxed);
        Status {
            max_size,
            size,
            available,
        }
    }

    /// Returns [`Manager`] of this [`Pool`].
    #[must_use]
    pub fn manager(&self) -> &M {
        &*self.inner.manager
    }
}

struct PoolInner<M: Manager> {
    manager: Box<M>,
    queue: Mutex<VecDeque<M::Type>>,
    size: AtomicUsize,
    /// Number of available [`Object`]s in the [`Pool`]. If there are no
    /// [`Object`]s in the [`Pool`] this number can become negative and store
    /// the number of [`Future`]s waiting for an [`Object`].
    available: AtomicIsize,
    semaphore: Semaphore,
    config: PoolConfig,
    runtime: Option<Runtime>,
    hooks: hooks::Hooks<M>,
}

// Implemented manually to avoid unnecessary trait bound on the struct.
impl<M> fmt::Debug for PoolInner<M>
where
    M: fmt::Debug + Manager,
    M::Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolInner")
            .field("manager", &self.manager)
            .field("queue", &self.queue)
            .field("size", &self.size)
            .field("available", &self.available)
            .field("semaphore", &self.semaphore)
            .field("config", &self.config)
            .field("runtime", &self.runtime)
            .field("hooks", &self.hooks)
            .finish()
    }
}

impl<M: Manager> PoolInner<M> {
    /// Cleans up internals of this [`Pool`].
    ///
    /// This method is called after closing the [`Pool`] and whenever an
    /// [`Object`] is returned to the [`Pool`] and makes sure closed [`Pool`]s
    /// don't contain any [`Object`]s.
    fn clean_up(&self) {
        if self.is_closed() {
            self.clear();
        }
    }

    /// Removes all the [`Object`]s which are currently part of this [`Pool`].
    fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        let _ = self.size.fetch_sub(queue.len(), Ordering::Relaxed);
        let _ = self
            .available
            .fetch_sub(queue.len() as isize, Ordering::Relaxed);
        queue.clear();
    }

    /// Indicates whether this [`Pool`] has been closed.
    fn is_closed(&self) -> bool {
        matches!(
            self.semaphore.try_acquire_many(0),
            Err(TryAcquireError::Closed)
        )
    }
}

async fn apply_timeout<O, E>(
    runtime: &Option<Runtime>,
    timeout_type: TimeoutType,
    duration: Option<Duration>,
    future: impl Future<Output = Result<O, impl Into<PoolError<E>>>>,
) -> Result<O, PoolError<E>> {
    match (runtime, duration) {
        (_, None) => future.await.map_err(Into::into),
        (Some(runtime), Some(duration)) => runtime
            .timeout(duration, future)
            .await
            .ok_or(PoolError::Timeout(timeout_type))?
            .map_err(Into::into),
        (None, Some(_)) => Err(PoolError::NoRuntimeSpecified),
    }
}
