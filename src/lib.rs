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
//!         "Hooray!".to_string()
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
//!     async fn recycle(&self, conn: Connection) -> Option<Connection> {
//!         if conn.check_health().await {
//!             Some(conn)
//!         } else {
//!             None
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
//!     assert_eq!(value, "Hooray!".to_string());
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
    /// Try to recycle an instance of `T` returning None` if the
    /// object could not be recycled.
    async fn recycle(&self, obj: T) -> Option<T>;
}

/// A wrapper around the actual pooled object which implements the traits
/// `Deref`, `DerefMut` and `Drop`. Use this object just as if it was of type
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
            pool: Arc::downgrade(&pool.inner)
        }
    }
}

impl<T, E> Drop for Object<T, E> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.upgrade() {
            pool.return_obj(self.obj.take());
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

/// This structure is used to undo changes that were done to the internal
/// pool state if a future executing `Pool::get` is aborted and dropped.
enum Undo {
    SizeAdd,
    AvailableSub,
    Recycle,
    None
}

struct UndoGuard<T, E> {
    pool: Weak<PoolInner<T, E>>,
    what: Undo,
}

impl<T, E> UndoGuard<T, E> {
    fn new(pool: &Pool<T, E>, what: Undo) -> Self {
        Self {
            pool: Arc::downgrade(&pool.inner),
            what
        }
    }
    pub fn abort(mut self) {
        self.what = Undo::None;
    }
}

impl<T, E> Drop for UndoGuard<T, E> {
    fn drop(&mut self) {
        if let Undo::None = self.what {
            return;
        }
        if let Some(pool) = self.pool.upgrade() {
            match self.what {
                Undo::SizeAdd => {
                    pool.size.fetch_sub(1, Ordering::Relaxed);
                }
                Undo::AvailableSub => {
                    pool.available.fetch_add(1, Ordering::Relaxed);
                }
                Undo::Recycle => {
                    if let Err(e) = pool.obj_sender.clone().try_send(None) {
                        // This code should be unreachable. Still if this ever
                        // happens fixing the fool state is a good idea.
                        pool.available.fetch_sub(1, Ordering::Relaxed);
                        pool.size.fetch_sub(1, Ordering::Relaxed);
                        if !std::thread::panicking() {
                            unreachable!("Could not return object to pool: {}", e);
                        }
                    }
                }
                Undo::None => {}
            }
        }
    }
}

struct PoolInner<T, E> {
    manager: Box<dyn Manager<T, E> + Sync + Send>,
    max_size: usize,
    obj_sender: Sender<Option<T>>,
    obj_receiver: Mutex<Receiver<Option<T>>>,
    size: AtomicUsize,
    /// The number of available objects in the pool. If there are no
    /// objects in the pool this number can become negative and stores the
    /// number of futures waiting for an object.
    available: AtomicIsize,
}

impl<T, E> PoolInner<T, E> {
    fn return_obj(&self, obj: Option<T>) {
        match self.obj_sender.clone().try_send(obj) {
            Ok(_) => {
                self.available.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => {
                // This code should be unreachable. Still if this ever
                // happens fixing the fool state is a good idea.
                self.size.fetch_sub(1, Ordering::Relaxed);
                if !std::thread::panicking() {
                    unreachable!("Could not return object to pool: {}", e);
                }
            }
        }
    }
}

/// A generic object and connection pool.
///
/// This struct can be cloned and transferred across thread boundaries
/// and uses reference counting for its internal state.
pub struct Pool<T, E> {
    inner: Arc<PoolInner<T, E>>,
}

#[derive(Debug)]
/// The current pool status.
pub struct Status {
    size: usize,
    available: isize,
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
        let (obj_sender, obj_receiver) = channel::<Option<T>>(max_size);
        Pool {
            inner: Arc::new(PoolInner {
                max_size: max_size,
                manager: Box::new(manager),
                obj_sender: obj_sender,
                obj_receiver: Mutex::new(obj_receiver),
                size: AtomicUsize::new(0),
                available: AtomicIsize::new(0)
            }),
        }
    }
    /// Retrieve object from pool or wait for one to become available.
    pub async fn get(&self) -> Result<Object<T, E>, E> {
        let mut available = self.inner.available.fetch_sub(1, Ordering::Relaxed);
        let undo_available = UndoGuard::new(&self, Undo::AvailableSub);
        let mut size = self.inner.size.load(Ordering::Relaxed);
        loop {
            if available <= 0 && size < self.inner.max_size {
                // The pool is empty and the max size has not been
                // reached, yet.
                let undo_size = UndoGuard::new(&self, Undo::SizeAdd);
                if self.inner.size.fetch_add(1, Ordering::Relaxed) < self.inner.max_size {
                    drop(undo_available);
                    let obj = self.inner.manager.create().await?;
                    undo_size.abort();
                    return Ok(Object::new(&self, obj));
                }
            }
            let obj = self.inner.obj_receiver.lock().await.recv().await.unwrap();
            if let Some(obj) = obj {
                let undo_recycle = UndoGuard::new(&self, Undo::Recycle);
                if let Some(obj) = self.inner.manager.recycle(obj).await {
                    undo_available.abort();
                    undo_recycle.abort();
                    return Ok(Object::new(&self, obj));
                }
                undo_recycle.abort();
            }
            // At this point either no object was received from the channel
            // or recycling the object failed.
            size = self.inner.size.fetch_sub(1, Ordering::Relaxed) - 1;
            available = self.inner.available.fetch_sub(1, Ordering::Relaxed);
        }
    }
    /// Retrieve status of the pool
    pub fn status(&self) -> Status {
        let size = self.inner.size.load(Ordering::Relaxed);
        let available = self.inner.available.load(Ordering::Relaxed);
        Status { size, available }
    }
}
