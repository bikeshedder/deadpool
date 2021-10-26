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
mod dropguard;
mod errors;
mod hooks;
mod metrics;
pub mod reexports;

#[deprecated(
    since = "0.9.1",
    note = "This module has been deprecated in favor of the dedicated `deadpool-sync` utility crate."
)]
pub mod sync;

use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{
        atomic::{AtomicIsize, Ordering},
        Arc, Mutex, Weak,
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use deadpool_runtime::Runtime;
use tokio::sync::{Semaphore, TryAcquireError};

pub use crate::Status;

use self::dropguard::DropGuard;
pub use self::{
    builder::{BuildError, PoolBuilder},
    config::{CreatePoolError, PoolConfig, Timeouts},
    errors::{PoolError, RecycleError, TimeoutType},
    hooks::{Hook, HookError, HookErrorCause, HookFuture, HookResult},
    metrics::Metrics,
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

/// Wrapper around the actual pooled object which implements [`Deref`],
/// [`DerefMut`] and [`Drop`] traits.
///
/// Use this object just as if it was of type `T` and upon leaving a scope the
/// [`Drop::drop()`] will take care of returning it to the pool.
#[must_use]
pub struct Object<M: Manager> {
    inner: Option<ObjectInner<M>>,

    /// This flag is used by the `Drop` implementation to decide
    /// wether to return the object to the pool or detach it.
    ready: bool,
}

impl<M> fmt::Debug for Object<M>
where
    M: fmt::Debug + Manager,
    M::Type: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Object")
            .field("inner", &self.inner)
            .finish()
    }
}

#[derive(Debug)]
pub(crate) struct ObjectInner<M: Manager> {
    /// Actual pooled object.
    obj: M::Type,

    /// Object metrics.
    metrics: Metrics,

    /// Pool to return the pooled object to.
    pool: Weak<PoolInner<M>>,
}

impl<M: Manager> Object<M> {
    /// Takes this [`Object`] from its [`Pool`] permanently. This reduces the
    /// size of the [`Pool`].
    #[must_use]
    pub fn take(mut this: Self) -> M::Type {
        if let Some(pool) = Object::pool(&this) {
            pool.inner
                .detach_object(&mut this.inner.as_mut().unwrap().obj)
        }
        this.inner.take().unwrap().obj
    }

    /// Get object statistics
    pub fn metrics(this: &Self) -> &Metrics {
        &this.inner.as_ref().unwrap().metrics
    }

    /// Returns the [`Pool`] this [`Object`] belongs to.
    ///
    /// Since [`Object`]s only hold a [`Weak`] reference to the [`Pool`] they
    /// come from, this can fail and return [`None`] instead.
    pub fn pool(this: &Self) -> Option<Pool<M>> {
        this.inner
            .as_ref()
            .unwrap()
            .pool
            .upgrade()
            .map(|inner| Pool {
                inner,
                _wrapper: PhantomData::default(),
            })
    }
}

impl<M: Manager> Drop for Object<M> {
    fn drop(&mut self) {
        if let Some(mut inner) = self.inner.take() {
            if let Some(pool) = inner.pool.upgrade() {
                if self.ready {
                    pool.return_object(inner)
                } else {
                    pool.detach_object(&mut inner.obj)
                }
            }
        }
    }
}

impl<M: Manager> Deref for Object<M> {
    type Target = M::Type;
    fn deref(&self) -> &M::Type {
        &self.inner.as_ref().unwrap().obj
    }
}

impl<M: Manager> DerefMut for Object<M> {
    fn deref_mut(&mut self) -> &mut M::Type {
        &mut self.inner.as_mut().unwrap().obj
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
                slots: Mutex::new(Slots {
                    vec: VecDeque::with_capacity(builder.config.max_size),
                    size: 0,
                    max_size: builder.config.max_size,
                }),
                available: AtomicIsize::new(0),
                semaphore: Semaphore::new(builder.config.max_size),
                config: builder.config,
                hooks: builder.hooks,
                runtime: builder.runtime,
            }),
            _wrapper: PhantomData::default(),
        }
    }

    /// Retrieves an [`Object`] from this [`Pool`] or waits for one to
    /// become available.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn get(&self) -> Result<W, PoolError<M::Error>> {
        self.timeout_get(&self.inner.config.timeouts).await
    }

    /// Retrieves an [`Object`] from this [`Pool`] and doesn't wait if there is
    /// currently no [`Object`] available and the maximum [`Pool`] size has
    /// been reached.
    ///
    /// # Errors
    ///
    /// See [`PoolError`] for details.
    pub async fn try_get(&self) -> Result<W, PoolError<M::Error>> {
        let mut timeouts = self.inner.config.timeouts;
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
        let available_guard = DropGuard(|| {
            let _ = self.inner.available.fetch_add(1, Ordering::Relaxed);
        });

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
                self.inner.runtime,
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

        let mut obj = loop {
            let mut obj = Object {
                inner: self.inner.slots.lock().unwrap().vec.pop_front(),
                ready: false,
            };
            if obj.inner.is_some() {
                // Recycle existing object
                let recycle_guard = DropGuard(|| {
                    // This guard is dropped either if the future is cancelled,
                    // the `Manager::recycle` method returns an error or the
                    // `pre_recycle`/`post_recycle` hooks return an error.
                    let _ = self.inner.available.fetch_sub(1, Ordering::Relaxed);
                });

                // Apply post_recycle hooks
                if let Some(_e) = self
                    .inner
                    .hooks
                    .pre_recycle
                    .apply(&mut obj.inner.as_mut().unwrap(), PoolError::PreRecycleHook)
                    .await?
                {
                    continue;
                }

                if apply_timeout(
                    self.inner.runtime,
                    TimeoutType::Recycle,
                    self.inner.config.timeouts.recycle,
                    self.inner.manager.recycle(&mut *obj),
                )
                .await
                .is_err()
                {
                    continue;
                }

                // Apply post_recycle hooks
                if let Some(_e) = self
                    .inner
                    .hooks
                    .post_recycle
                    .apply(&mut obj.inner.as_mut().unwrap(), PoolError::PostRecycleHook)
                    .await?
                {
                    continue;
                }

                obj.inner.as_mut().unwrap().metrics.recycle_count += 1;
                obj.inner.as_mut().unwrap().metrics.recycled = Some(Instant::now());

                recycle_guard.disarm();

                break obj;
            } else {
                // Create new object
                let mut obj = Object {
                    inner: Some(ObjectInner {
                        obj: apply_timeout(
                            self.inner.runtime,
                            TimeoutType::Create,
                            self.inner.config.timeouts.create,
                            self.inner.manager.create(),
                        )
                        .await?,
                        metrics: Metrics::default(),
                        pool: Arc::downgrade(&self.inner),
                    }),
                    ready: false,
                };

                // Apply post_create hooks
                if let Some(_e) = self
                    .inner
                    .hooks
                    .post_create
                    .apply(&mut obj.inner.as_mut().unwrap(), PoolError::PostCreateHook)
                    .await?
                {
                    continue;
                }

                self.inner.slots.lock().unwrap().size += 1;
                let _ = self.inner.available.fetch_add(1, Ordering::Relaxed);

                break obj;
            }
        };

        available_guard.disarm();
        permit.forget();

        obj.ready = true;
        Ok(obj.into())
    }

    /**
     * Resize the pool. This change the `max_size` of the pool dropping
     * excess objects and/or making space for new ones.
     *
     * If the pool is closed this method does nothing. The [`Pool::status`] method
     * always reports a `max_size` of 0 for closed pools.
     */
    pub fn resize(&self, max_size: usize) {
        if self.inner.semaphore.is_closed() {
            return;
        }
        let mut slots = self.inner.slots.lock().unwrap();
        let old_max_size = slots.max_size;
        slots.max_size = max_size;
        // shrink pool
        if max_size < old_max_size {
            while slots.size > slots.max_size {
                if let Ok(permit) = self.inner.semaphore.try_acquire() {
                    permit.forget();
                    if slots.vec.pop_front().is_some() {
                        slots.size -= 1;
                    }
                } else {
                    break;
                }
            }
            // Create a new VecDeque with a smaller capacity
            let mut vec = VecDeque::with_capacity(max_size);
            for obj in slots.vec.drain(..) {
                vec.push_back(obj);
            }
            slots.vec = vec;
        }
        // grow pool
        if max_size > old_max_size {
            let additional = slots.max_size - slots.size;
            slots.vec.reserve_exact(additional);
            self.inner.semaphore.add_permits(additional);
        }
    }

    /// Closes this [`Pool`].
    ///
    /// All current and future tasks waiting for [`Object`]s will return
    /// [`PoolError::Closed`] immediately.
    ///
    /// This operation resizes the pool to 0.
    pub fn close(&self) {
        self.resize(0);
        self.inner.semaphore.close();
    }

    /// Indicates whether this [`Pool`] has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.semaphore.is_closed()
    }

    /// Retrieves [`Status`] of this [`Pool`].
    #[must_use]
    pub fn status(&self) -> Status {
        let slots = self.inner.slots.lock().unwrap();
        let available = self.inner.available.load(Ordering::Relaxed);
        Status {
            max_size: slots.max_size,
            size: slots.size,
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
    slots: Mutex<Slots<ObjectInner<M>>>,
    /// Number of available [`Object`]s in the [`Pool`]. If there are no
    /// [`Object`]s in the [`Pool`] this number can become negative and store
    /// the number of [`Future`]s waiting for an [`Object`].
    available: AtomicIsize,
    semaphore: Semaphore,
    config: PoolConfig,
    runtime: Option<Runtime>,
    hooks: hooks::Hooks<M>,
}

#[derive(Debug)]
struct Slots<T> {
    vec: VecDeque<T>,
    size: usize,
    max_size: usize,
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
            .field("slots", &self.slots)
            .field("available", &self.available)
            .field("semaphore", &self.semaphore)
            .field("config", &self.config)
            .field("runtime", &self.runtime)
            .field("hooks", &self.hooks)
            .finish()
    }
}

impl<M: Manager> PoolInner<M> {
    fn return_object(&self, mut inner: ObjectInner<M>) {
        let mut slots = self.slots.lock().unwrap();
        if slots.size <= slots.max_size {
            slots.vec.push_back(inner);
            drop(slots);
            let _ = self.available.fetch_add(1, Ordering::Relaxed);
            self.semaphore.add_permits(1);
        } else {
            slots.size -= 1;
            drop(slots);
            self.manager.detach(&mut inner.obj);
        }
    }
    fn detach_object(&self, obj: &mut M::Type) {
        let mut slots = self.slots.lock().unwrap();
        let add_permits = slots.size <= slots.max_size;
        slots.size -= 1;
        drop(slots);
        if add_permits {
            self.semaphore.add_permits(1);
        }
        self.manager.detach(obj);
    }
}

async fn apply_timeout<O, E>(
    runtime: Option<Runtime>,
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
