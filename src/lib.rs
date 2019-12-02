//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! # Example
//!
//! ```rust
//! use async_trait::async_trait;
//!
//! #[derive(Debug)]
//! enum Error { Fail }
//!
//! struct Connection {}
//!
//! type Pool = deadpool::Pool<Connection, Error>;
//!
//! impl Connection {
//!     async fn new() -> Result<Self, Error> {
//!         Ok(Connection {})
//!     }
//!     async fn check_health(&self) -> bool {
//!         true
//!     }
//!     async fn do_something(&self) -> String {
//!         "Horray!".to_string()
//!     }
//! }
//!
//! struct Manager {}
//!
//! #[async_trait]
//! impl deadpool::Manager<Connection, Error> for Manager
//! {
//!     async fn create(&self) -> Result<Connection, Error> {
//!         Connection::new().await
//!     }
//!     async fn recycle(&self, conn: Connection) -> Result<Connection, Error> {
//!         if conn.check_health().await {
//!             Ok(conn)
//!         } else {
//!             Connection::new().await
//!         }
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let mgr = Manager {};
//!     let pool = Pool::new(mgr, 16);
//!     let mut conn = pool.get().await.unwrap();
//!     let value = conn.do_something().await;
//!     assert_eq!(value, "Horray!".to_string());
//! }
//! ```
//!
//! For a more complete example please see
//! [`deadpool-postgres`](https://crates.io/crates/deadpool-postgres)
#![warn(missing_docs)]

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

#[cfg(feature = "postgres")]
pub mod postgres;

/// This trait is used to `create` new objects or `recycle` existing ones.
#[async_trait]
pub trait Manager<T, E> {
    /// Create a new instance of `T`
    async fn create(&self) -> Result<T, E>;
    /// Try to recycle an instance of `T`, create a new instance of `T` if
    /// that fails or return an error if that fails, too.
    async fn recycle(&self, obj: T) -> Result<T, E>;
}

/// A wrapper around the actual pooled object which implements the traits
/// `Deref`, `DerefMut` and `Drop`. Use this object just as it was of type
/// `T` and upon leaving scope the `drop` function will take care of
/// returning it to the pool.
pub struct Object<T, E> {
    obj: Option<T>,
    pool: Weak<PoolInner<T, E>>,
}

impl<T, E> Object<T, E> {
    fn new(pool: &Pool<T, E>, obj: T) -> Object<T, E> {
        Object {
            obj: Some(obj),
            pool: Arc::downgrade(&pool.inner),
        }
    }
}

impl<T, E> Drop for Object<T, E> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            pool.return_obj(self.obj.take().unwrap());
        }
    }
}

impl<T, E> Deref for Object<T, E> {
    type Target = T;
    fn deref(&self) -> &T {
        self.obj.as_ref().unwrap()
    }
}

impl<T, E> DerefMut for Object<T, E> {
    fn deref_mut(&mut self) -> &mut T {
        self.obj.as_mut().unwrap()
    }
}

#[derive(Default)]
struct PoolSize {
    current: AtomicUsize,
    available: AtomicIsize,
}

struct PoolInner<T, E> {
    manager: Box<dyn Manager<T, E> + Sync + Send>,
    max_size: usize,
    obj_sender: Sender<T>,
    obj_receiver: Mutex<Receiver<T>>,
    size: PoolSize,
}

impl<T, E> PoolInner<T, E> {
    fn return_obj(&self, obj: T) {
        self.size.available.fetch_add(1, Ordering::SeqCst);
        self.obj_sender
            .clone()
            .try_send(obj)
            .map_err(|_| ())
            .unwrap();
    }
}

/// A generic object and connection pool.
///
/// This struct can be cloned and transferred accross thread boundaries
/// and uses reference counting for its internal state.
pub struct Pool<T, E> {
    inner: Arc<PoolInner<T, E>>,
}

impl<T, E> Clone for Pool<T, E> {
    fn clone(&self) -> Pool<T, E> {
        Pool {
            inner: self.inner.clone(),
        }
    }
}

impl<T, E> Pool<T, E> {
    /// Create new connection pool with a given `manager` and `max_size`.
    /// The `manager` is used to create and recycle objects and `max_size`
    /// is the maximum number of objects ever created.
    pub fn new(manager: impl Manager<T, E> + Send + Sync + 'static, max_size: usize) -> Pool<T, E> {
        let (obj_sender, obj_receiver) = channel::<T>(max_size);
        Pool {
            inner: Arc::new(PoolInner {
                max_size: max_size,
                manager: Box::new(manager),
                obj_sender: obj_sender,
                obj_receiver: Mutex::new(obj_receiver),
                size: PoolSize::default(),
            }),
        }
    }
    /// Retrieve object from pool or wait for one to become available.
    pub async fn get(&self) -> Result<Object<T, E>, E> {
        let available = self.inner.size.available.fetch_sub(1, Ordering::SeqCst);
        if available <= 0 && self.inner.size.current.load(Ordering::SeqCst) < self.inner.max_size {
            let current = self.inner.size.current.fetch_add(1, Ordering::SeqCst);
            if current < self.inner.max_size {
                self.inner.size.available.fetch_add(1, Ordering::SeqCst);
                let obj = self.inner.manager.create().await?;
                return Ok(Object::new(&self, obj));
            }
        }
        let obj = self.inner.obj_receiver.lock().await.recv().await.unwrap();
        let obj = self.inner.manager.recycle(obj).await?;
        Ok(Object::new(&self, obj))
    }
}
