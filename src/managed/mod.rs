//! This module contains the managed version of the pool. Managed meaning
//! that it requires a `Manager` trait which is responsible for creating
//! and recycling objects as they are needed.
//!
//! # Example
//!
//! ```rust
//! use async_trait::async_trait;
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
//! impl deadpool::managed::Manager for Manager {
//!     type Type = Computer;
//!     type Error = Error;
//!     async fn create(&self) -> Result<Computer, Error> {
//!         Ok(Computer {})
//!     }
//!     async fn recycle(&self, conn: &mut Computer) -> deadpool::managed::RecycleResult<Error> {
//!         Ok(())
//!     }
//! }
//!
//! type Pool = deadpool::managed::Pool<Manager>;
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
//! [`deadpool-postgres`](https://crates.io/crates/deadpool-postgres)

use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;
use std::{future::Future, marker::PhantomData};

use async_trait::async_trait;
use tokio::sync::{Semaphore, TryAcquireError};

mod builder;
pub use builder::{BuildError, PoolBuilder};
mod config;
pub use self::config::{PoolConfig, Timeouts};
mod errors;
pub use errors::{PoolError, RecycleError, TimeoutType};
pub mod sync;

use crate::runtime::Runtime;
pub use crate::Status;

/// Result type for the recycle function
pub type RecycleResult<E> = Result<(), RecycleError<E>>;

/// This trait is used to `create` new objects or `recycle` existing ones.
#[async_trait]
pub trait Manager: Sync + Send {
    /// Type that the manager creates and recycles.
    type Type;
    /// The error that the manager can return when creating and recycling
    /// objects.
    type Error;
    /// Create a new instance of `Type`
    async fn create(&self) -> Result<Self::Type, Self::Error>;
    /// Try to recycle an instance of `Type` returning an `Error` if the
    /// object could not be recycled.
    async fn recycle(&self, obj: &mut Self::Type) -> RecycleResult<Self::Error>;
    /// Detach an instance of `Type` from this manager. This method is
    /// called when using the `Object::take` function for removing
    /// an object from the pool. If the manager doesn't hold any
    /// references to the handed out objects the default implementation
    /// can be used which does nothing.
    fn detach(&self, _obj: &mut Self::Type) {}
}

enum ObjectState {
    Waiting,
    Receiving,
    Creating,
    Recycling,
    Ready,
    Taken,
    Dropped,
}

/// A wrapper around the actual pooled object which implements the traits
/// `Deref`, `DerefMut` and `Drop`. Use this object just as if it was of type
/// `T` and upon leaving scope the `drop` function will take care of
/// returning it to the pool.
pub struct Object<M: Manager> {
    obj: Option<M::Type>,
    state: ObjectState,
    pool: Weak<PoolInner<M>>,
}

impl<M: Manager> Object<M> {
    /// Take this object from the pool permanently. This reduces the size of
    /// the pool.
    pub fn take(mut this: Self) -> M::Type {
        this.state = ObjectState::Taken;
        if let Some(pool) = this.pool.upgrade() {
            pool.manager.detach(&mut this);
        }
        this.obj.take().unwrap()
    }
    /// Get the pool this object belongs to. Since objects only hold a
    /// weak reference to the pool they come from this can fail and
    /// return `None` instead.
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
                    pool.available.fetch_add(1, Ordering::Relaxed);
                }
                ObjectState::Receiving => {
                    pool.available.fetch_add(1, Ordering::Relaxed);
                    pool.semaphore.add_permits(1);
                }
                ObjectState::Creating | ObjectState::Taken => {
                    pool.size.fetch_sub(1, Ordering::Relaxed);
                    pool.semaphore.add_permits(1);
                }
                ObjectState::Recycling | ObjectState::Ready => {
                    pool.available.fetch_add(1, Ordering::Relaxed);
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

struct PoolInner<M: Manager> {
    manager: Box<M>,
    queue: std::sync::Mutex<VecDeque<M::Type>>,
    size: AtomicUsize,
    /// The number of available objects in the pool. If there are no
    /// objects in the pool this number can become negative and stores the
    /// number of futures waiting for an object.
    available: AtomicIsize,
    semaphore: Semaphore,
    config: PoolConfig,
    runtime: Option<Runtime>,
}

/// A generic object and connection pool.
///
/// This struct can be cloned and transferred across thread boundaries
/// and uses reference counting for its internal state.
pub struct Pool<M: Manager, W: From<Object<M>> = Object<M>> {
    inner: Arc<PoolInner<M>>,
    _wrapper: PhantomData<fn() -> W>,
}

impl<M: Manager, W: From<Object<M>>> Clone for Pool<M, W> {
    fn clone(&self) -> Pool<M, W> {
        Pool {
            inner: self.inner.clone(),
            _wrapper: PhantomData::default(),
        }
    }
}

impl<M: Manager, W: From<Object<M>>> Pool<M, W> {
    /// Create a new pool builder. This is the only way to create a pool
    /// instance.
    pub fn builder(manager: M) -> PoolBuilder<M, W> {
        PoolBuilder::new(manager)
    }
    pub(crate) fn from_builder(builder: PoolBuilder<M, W>) -> Pool<M, W> {
        Pool {
            inner: Arc::new(PoolInner {
                manager: Box::new(builder.manager),
                queue: std::sync::Mutex::new(VecDeque::with_capacity(builder.config.max_size)),
                size: AtomicUsize::new(0),
                available: AtomicIsize::new(0),
                semaphore: Semaphore::new(builder.config.max_size),
                config: builder.config,
                runtime: builder.runtime,
            }),
            _wrapper: PhantomData::default(),
        }
    }
    /// Retrieve object from pool or wait for one to become available.
    pub async fn get(&self) -> Result<W, PoolError<M::Error>> {
        self.timeout_get(&self.inner.config.timeouts).await
    }
    /// Retrieve object from the pool and do not wait if there is currently
    /// no object available and the maximum pool size has been reached.
    pub async fn try_get(&self) -> Result<W, PoolError<M::Error>> {
        let mut timeouts = self.inner.config.timeouts.clone();
        timeouts.wait = Some(Duration::from_secs(0));
        self.timeout_get(&timeouts).await
    }
    /// Retrieve object using a different timeout config than the one
    /// configured.
    pub async fn timeout_get(&self, timeouts: &Timeouts) -> Result<W, PoolError<M::Error>> {
        self.inner.available.fetch_sub(1, Ordering::Relaxed);

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
                        Ok(_) => break,
                        Err(_) => {
                            self.inner.available.fetch_sub(1, Ordering::Relaxed);
                            self.inner.size.fetch_sub(1, Ordering::Relaxed);
                            continue;
                        }
                    }
                }
                None => {
                    // Create new object
                    obj.state = ObjectState::Creating;
                    self.inner.available.fetch_add(1, Ordering::Relaxed);
                    self.inner.size.fetch_add(1, Ordering::Relaxed);
                    obj.obj = Some(
                        apply_timeout(
                            &self.inner.runtime,
                            TimeoutType::Create,
                            self.inner.config.timeouts.create,
                            self.inner.manager.create(),
                        )
                        .await?,
                    );
                    break;
                }
            }
        }

        obj.state = ObjectState::Ready;
        Ok(obj.into())
    }
    /// Close the pool
    ///
    /// All current and future tasks waiting for objects return
    /// `Err(PoolError::Closed)` immediately.
    pub fn close(&self) {
        self.inner.semaphore.close();
        self.inner.clean_up();
    }
    /// Returns true if the pool has been closed
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
    /// Retrieve status of the pool
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
    /// Get manager of the pool
    pub fn manager(&self) -> &M {
        &*self.inner.manager
    }
}

impl<M: Manager> PoolInner<M> {
    /// Clean up internals of the pool.
    ///
    /// This method is called after closing the pool and whenever a
    /// object is returned to the pool and makes sure closed pools
    /// do not contain
    fn clean_up(&self) {
        if self.is_closed() {
            self.clear();
        }
    }
    /// Remove all objects which are currently part of the pool.
    fn clear(&self) {
        let mut queue = self.queue.lock().unwrap();
        self.size.fetch_sub(queue.len(), Ordering::Relaxed);
        self.available
            .fetch_sub(queue.len() as isize, Ordering::Relaxed);
        queue.clear();
    }
    /// Returns true if the pool has been closed
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
            .ok_or_else(|| PoolError::Timeout(timeout_type))?
            .map_err(Into::into),
        (None, Some(_)) => Err(PoolError::NoRuntimeSpecified),
    }
}
