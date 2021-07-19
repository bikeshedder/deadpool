//! This module contains helpers when writing pools for objects
//! that don't support async and need to be run inside a thread.

use std::{
    any::Any,
    fmt,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crate::{runtime::SpawnBlockingError, Runtime};

/// This error is returned when the [`Connection::interact`] call
/// fails.
#[derive(Debug)]
pub enum InteractError<E> {
    /// The provided callback panicked
    Panic(Box<dyn Any + Send + 'static>),
    /// The callback was aborted
    Aborted,
    /// backend returned an error
    Backend(E),
}

impl<E> fmt::Display for InteractError<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic(_) => write!(f, "Panic"),
            Self::Aborted => write!(f, "Aborted"),
            Self::Backend(e) => write!(f, "Backend error: {}", e),
        }
    }
}

impl<E> std::error::Error for InteractError<E> where E: std::error::Error {}

/// A wrapper for objects which only provides blocking functions that
/// need to be called on a separate thread. This wrapper provides access
/// to the wrapped object via the `interact` function.
pub struct SyncWrapper<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    obj: Arc<Mutex<Option<T>>>,
    runtime: Runtime,
    _error: PhantomData<fn() -> E>,
}

impl<T, E> SyncWrapper<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Create a new wrapped object.
    ///
    /// This wrapper requires an actual runtime and therefore
    /// this method returns `None` when passing `Runtime::None`
    /// as runtime.
    pub async fn new<F>(runtime: Runtime, f: F) -> Option<Result<Self, E>>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
    {
        let result = match runtime.spawn_blocking(move || f()).await {
            Err(SpawnBlockingError::NoRuntime) => return None,
            // FIXME panicking when the creation panics is not nice
            // In order to handle this properly the Manager::create
            // methods needs to support a custom error enum which
            // supports a Panic variant.
            Err(SpawnBlockingError::Panic(e)) => panic!("{:?}", e),
            Ok(obj) => obj,
        };
        Some(result.map(|obj| Self {
            obj: Arc::new(Mutex::new(Some(obj))),
            runtime,
            _error: PhantomData::default(),
        }))
    }
    /// Interact with the underlying object. This function expects a closure
    /// that takes the object as parameter. The closure is executed in a
    /// separate thread so that the async runtime is not blocked.
    pub async fn interact<F, R>(&self, f: F) -> Result<R, InteractError<E>>
    where
        F: FnOnce(&T) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
    {
        let arc = self.obj.clone();
        self.runtime
            .spawn_blocking(move || {
                let guard = arc.lock().unwrap();
                let conn = guard.as_ref().unwrap();
                f(&conn)
            })
            .await
            .map_err(|e| match e {
                SpawnBlockingError::NoRuntime => unreachable!(),
                SpawnBlockingError::Panic(p) => InteractError::Panic(p),
            })?
            .map_err(InteractError::Backend)
    }
    /// Returns if the underlying mutex has been poisoned.
    /// This happens when a panic occurs while interacting with
    /// the object.
    pub fn is_mutex_poisoned(&self) -> bool {
        self.obj.is_poisoned()
    }
}

impl<T, E> Drop for SyncWrapper<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    fn drop(&mut self) {
        let arc = self.obj.clone();
        // Drop the `rusqlite::Connection` inside a `spawn_blocking`
        // as the `drop` function of it can block.
        self.runtime
            .spawn_blocking_background(move || match arc.lock() {
                Ok(mut guard) => drop(guard.take()),
                Err(e) => drop(e.into_inner().take()),
            })
            .unwrap();
    }
}
