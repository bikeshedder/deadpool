//! This module contains the unmanaged version of the pool. Unmanaged meaning
//! that no manager is used to create and recycle objects. Objects either need
//! to be created upfront or by adding them using the `add` or `try_add`
//! methods.
//!
//! # Example
//!
//! ```rust
//! use deadpool::unmanaged::Pool;
//!
//! struct Computer {}
//!
//! impl Computer {
//!     async fn get_answer(&self) -> i32 {
//!         42
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let pool = Pool::from(vec![
//!         Computer {},
//!         Computer {},
//!     ]);
//!     let s = pool.get().await.unwrap();
//!     assert_eq!(s.get_answer().await, 42);
//! }
//! ```

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::{convert::TryInto, time::Duration};

use tokio::sync::{Semaphore, TryAcquireError};

use crate::runtime::TimeoutError;
pub use crate::Status;

mod config;
pub use self::config::PoolConfig;
mod errors;
pub use self::errors::PoolError;

/// A wrapper around the actual pooled object which implements the traits
/// `Deref`, `DerefMut` and `Drop`. Use this object just as if it was of type
/// `T` and upon leaving scope the `drop` function will take care of
/// returning it to the pool.
pub struct Object<T> {
    obj: Option<T>,
    pool: Weak<PoolInner<T>>,
}

impl<T> Object<T> {
    /// Take this object from the pool permanently. This reduces the size of
    /// the pool. If needed the object can later be added back to the pool
    /// using the `Pool::add` or `Pool::try_add` methods.
    pub fn take(mut this: Self) -> T {
        if let Some(pool) = this.pool.upgrade() {
            pool.size.fetch_sub(1, Ordering::Relaxed);
            pool.size_semaphore.add_permits(1);
        }
        this.obj.take().unwrap()
    }
}

impl<T> Drop for Object<T> {
    fn drop(&mut self) {
        if let Some(obj) = self.obj.take() {
            if let Some(pool) = self.pool.upgrade() {
                {
                    let mut queue = pool.queue.lock().unwrap();
                    queue.push(obj);
                }
                pool.available.fetch_add(1, Ordering::Relaxed);
                pool.semaphore.add_permits(1);
                pool.clean_up();
            }
        }
    }
}

impl<T> Deref for Object<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.obj.as_ref().unwrap()
    }
}

impl<T> DerefMut for Object<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.obj.as_mut().unwrap()
    }
}

impl<T> AsRef<T> for Object<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for Object<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

struct PoolInner<T> {
    config: PoolConfig,
    queue: Mutex<Vec<T>>,
    size: AtomicUsize,
    /// This semaphore has as many permits as `max_size - size`. Every time
    /// an object is added to the pool a permit is removed from the semaphore
    /// and every time an object is removed a permit is added back.
    size_semaphore: Semaphore,
    /// The number of available objects in the pool. If there are no
    /// objects in the pool this number can become negative and stores the
    /// number of futures waiting for an object.
    available: AtomicIsize,
    semaphore: Semaphore,
}

/// A generic object and connection pool. This is the static version of the
/// pool which does not include
///
/// This struct can be cloned and transferred across thread boundaries
/// and uses reference counting for its internal state.
///
/// A pool of existing objects can be created from an existing collection
/// of objects if it has a known exact size:
///
/// ```rust
/// use deadpool::unmanaged::Pool;
/// let pool = Pool::from(vec![1, 2, 3]);
/// ```
pub struct Pool<T> {
    inner: Arc<PoolInner<T>>,
}

impl<T> Clone for Pool<T> {
    fn clone(&self) -> Pool<T> {
        Pool {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::from_config(&PoolConfig::default())
    }
}

impl<T> Pool<T> {
    /// Create a new empty pool with the given max_size.
    pub fn new(max_size: usize) -> Self {
        Self::from_config(&PoolConfig::new(max_size))
    }
    /// Create a new empty pool using the given configuration
    pub fn from_config(config: &PoolConfig) -> Self {
        Self {
            inner: Arc::new(PoolInner {
                config: config.clone(),
                queue: Mutex::new(Vec::with_capacity(config.max_size)),
                size: AtomicUsize::new(0),
                size_semaphore: Semaphore::new(config.max_size),
                available: AtomicIsize::new(0),
                semaphore: Semaphore::new(0),
            }),
        }
    }
    /// Retrieve object from pool or wait for one to become available.
    pub async fn get(&self) -> Result<Object<T>, PoolError> {
        self.timeout_get(self.inner.config.timeout).await
    }
    /// Retrieve object from the pool and do not wait if there is currently
    /// no object available and the maximum pool size has been reached.
    pub fn try_get(&self) -> Result<Object<T>, PoolError> {
        let inner = self.inner.as_ref();
        let permit = inner.semaphore.try_acquire().map_err(|e| match e {
            TryAcquireError::NoPermits => PoolError::Timeout,
            TryAcquireError::Closed => PoolError::Closed,
        })?;
        let obj = {
            let mut queue = inner.queue.lock().unwrap();
            queue.pop().unwrap()
        };
        permit.forget();
        inner.available.fetch_sub(1, Ordering::Relaxed);
        Ok(Object {
            pool: Arc::downgrade(&self.inner),
            obj: Some(obj),
        })
    }
    /// Retrieve object using a different timeout config than the one
    /// configured.
    pub async fn timeout_get(&self, timeout: Option<Duration>) -> Result<Object<T>, PoolError> {
        let inner = self.inner.as_ref();
        let permit = match timeout {
            Some(timeout) if timeout.as_nanos() == 0 => {
                inner.semaphore.try_acquire().map_err(|e| match e {
                    TryAcquireError::NoPermits => PoolError::Timeout,
                    TryAcquireError::Closed => PoolError::Closed,
                })
            }
            Some(timeout) => match inner
                .config
                .runtime
                .timeout(timeout, inner.semaphore.acquire())
                .await
            {
                Ok(result) => result.map_err(|_| PoolError::Closed),
                Err(e) => Err(match e {
                    TimeoutError::NoRuntime => PoolError::NoRuntimeSpecified,
                    TimeoutError::Timeout => PoolError::Timeout,
                }),
            },
            None => inner
                .semaphore
                .acquire()
                .await
                .map_err(|_| PoolError::Closed),
        }?;
        let obj = {
            let mut queue = inner.queue.lock().unwrap();
            queue.pop().unwrap()
        };
        permit.forget();
        inner.available.fetch_sub(1, Ordering::Relaxed);
        Ok(Object {
            pool: Arc::downgrade(&self.inner),
            obj: Some(obj),
        })
    }
    /// Add object to pool. If the `size` has already reached `max_size`
    /// this function blocks until the object can be added to the pool.
    /// If the pool has been closed a tuple containing the object and
    /// the error is returned instead.
    pub async fn add(&self, obj: T) -> Result<(), (T, PoolError)> {
        match self.inner.size_semaphore.acquire().await {
            Ok(permit) => {
                permit.forget();
                self._add(obj);
                Ok(())
            }
            Err(_) => Err((obj, PoolError::Closed)),
        }
    }
    /// Try to add a pool to the object. If the `size` has already reached
    /// `max_size` or the pool has been closed a tuple containing the object
    /// and the cause of the error is returned instead.
    pub fn try_add(&self, obj: T) -> Result<(), (T, PoolError)> {
        match self.inner.size_semaphore.try_acquire() {
            Ok(permit) => {
                permit.forget();
                self._add(obj);
                Ok(())
            }
            Err(e) => Err(match e {
                TryAcquireError::NoPermits => (obj, PoolError::Timeout),
                TryAcquireError::Closed => (obj, PoolError::Closed),
            }),
        }
    }
    /// Internal function which adds an object to the pool. Prior calling
    /// this it must be guaranteed that `size` does not exceed `max_size`.
    /// In the methods `add` and `try_add` this is ensured by using the
    /// `size_semaphore`.
    fn _add(&self, obj: T) {
        self.inner.size.fetch_add(1, Ordering::Relaxed);
        {
            let mut queue = self.inner.queue.lock().unwrap();
            queue.push(obj);
        }
        self.inner.available.fetch_add(1, Ordering::Relaxed);
        self.inner.semaphore.add_permits(1);
    }
    /// Remove an object from the pool. This is a shortcut for
    /// ```rust,ignore
    /// Object::take(pool.get()?.await)
    /// ```
    pub async fn remove(&self) -> Result<T, PoolError> {
        self.get().await.map(Object::take)
    }
    /// Try to remove an object from the pool. This is a shortcut for
    /// ```rust,ignore
    /// if let Some(obj) = self.try_get() {
    ///     Some(Object::take(obj))
    /// } else {
    ///     None
    /// }
    /// ```
    pub fn try_remove(&self) -> Result<T, PoolError> {
        self.try_get().map(Object::take)
    }
    /// Remove object using a different timeout config than the one
    /// configured. This is a shortcut for
    /// ```rust,ignore
    /// Object::take(pool.timeout_get()?.await)
    /// ```
    pub async fn tiemout_remove(&self, timeout: Option<Duration>) -> Result<T, PoolError> {
        self.timeout_get(timeout).await.map(Object::take)
    }
    /// Close the pool
    ///
    /// All current and future tasks waiting for objects return
    /// `Err(PoolError::Closed)` immediately.
    pub fn close(&self) {
        self.inner.semaphore.close();
        self.inner.size_semaphore.close();
        self.inner.clear();
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
}

impl<T> PoolInner<T> {
    /// Clean up internals of the pool.
    ///
    /// This method is called after closing the pool and whenever a
    /// object is returned to the pool and makes sure closed pools
    /// do not contain objects.
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
    fn is_closed(&self) -> bool {
        matches!(
            self.semaphore.try_acquire_many(0),
            Err(TryAcquireError::Closed)
        )
    }
}

impl<T, I> From<I> for Pool<T>
where
    I: IntoIterator<Item = T>,
    <I as IntoIterator>::IntoIter: ExactSizeIterator,
{
    /// Create new pool from the given exact size iterator of objects.
    fn from(iter: I) -> Pool<T> {
        let queue = iter.into_iter().collect::<Vec<_>>();
        let len = queue.len();
        Pool {
            inner: Arc::new(PoolInner {
                queue: Mutex::new(queue),
                config: PoolConfig::new(len),
                size: AtomicUsize::new(len),
                size_semaphore: Semaphore::new(0),
                available: AtomicIsize::new(len.try_into().unwrap()),
                semaphore: Semaphore::new(len),
            }),
        }
    }
}
