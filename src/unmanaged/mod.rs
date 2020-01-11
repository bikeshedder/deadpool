//! This module contains the unmanaged version of the pool. Unmanaged meaning
//! that no manager is used to create and recycle objects and all the objects
//! need to be created upfront.
//!
//! # Example
//!
//! ```rust
//! use deadpool::unmanaged::Pool;
//!
//! #[tokio::main]
//! async fn main() {
//!     let pool = Pool::new(vec![
//!         "foo".to_string(),
//!         "bar".to_string(),
//!     ]);
//!     {
//!         let s = pool.get().await;
//!         assert_eq!(s.len(), 3);
//!         assert_eq!(pool.status().available, 1)
//!     }
//!     assert_eq!(pool.status().available, 2);
//! }
//! ```

use std::convert::TryInto;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Weak};

use crossbeam_queue::ArrayQueue;
use tokio::sync::Semaphore;

pub use crate::Status;

/// A wrapper around the actual pooled object which implements the traits
/// `Deref`, `DerefMut` and `Drop`. Use this object just as if it was of type
/// `T` and upon leaving scope the `drop` function will take care of
/// returning it to the pool.
pub struct Object<T> {
    obj: Option<T>,
    pool: Weak<PoolInner<T>>,
}

impl<T> Drop for Object<T> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            pool.queue.push(self.obj.take().unwrap()).unwrap();
            pool.available.fetch_add(1, Ordering::Relaxed);
            pool.semaphore.add_permits(1);
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

struct PoolInner<T> {
    queue: ArrayQueue<T>,
    size: usize,
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

impl<T> Pool<T> {
    /// Create new pool from the given exact size iterator of objects.
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=T>,
        <I as IntoIterator>::IntoIter: ExactSizeIterator,
    {
        let iter = iter.into_iter();
        let size = iter.len();
        let queue = ArrayQueue::new(size);
        for obj in iter {
            queue.push(obj).unwrap();
        }
        Self {
            inner: Arc::new(PoolInner {
                queue,
                size,
                available: AtomicIsize::new(size.try_into().unwrap()),
                semaphore: Semaphore::new(size),
            }),
        }
    }
    /// Retrieve object from pool or wait for one to become available.
    pub async fn get(&self) -> Object<T> {
        let permit = self.inner.semaphore.acquire().await;
        let obj = self.inner.queue.pop().unwrap();
        permit.forget();
        self.inner.available.fetch_sub(1, Ordering::Relaxed);
        Object {
            pool: Arc::downgrade(&self.inner),
            obj: Some(obj),
        }
    }
    /// Retrieve object from the pool and do not wait if there is currently
    /// no object available and the maximum pool size has been reached.
    pub fn try_get(&self) -> Option<Object<T>> {
        let permit = self.inner.semaphore.try_acquire().ok()?;
        let obj = self.inner.queue.pop().unwrap();
        permit.forget();
        self.inner.available.fetch_sub(1, Ordering::Relaxed);
        Some(Object {
            pool: Arc::downgrade(&self.inner),
            obj: Some(obj),
        })
    }
    /// Retrieve status of the pool
    pub fn status(&self) -> Status {
        let size = self.inner.size;
        let available = self.inner.available.load(Ordering::Relaxed);
        Status { size, available }
    }
}
