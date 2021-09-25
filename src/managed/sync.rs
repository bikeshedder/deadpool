//! Helpers for writing pools for objects that don't support async and need to
//! be run inside a thread.

pub mod reexports {
    //! This module contains all things that should be reexported
    //! by backend implementations in order to avoid direct
    //! dependencies on the `deadpool` crate itself.
    //!
    //! This module is the variant that should be used by *sync*
    //! backends.
    //!
    //! Crates based on `deadpool::managed::sync` should include this line:
    //! ```rust
    //! pub use deadpool::managed::sync::reexports::*;
    //! ```

    pub use super::super::reexports::*;
    pub use super::{InteractError, SyncGuard};
}

use std::{
    any::Any,
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError},
};

use crate::{runtime::SpawnBlockingError, Runtime};

/// Possible errors returned when [`SyncWrapper::interact()`] fails.
#[derive(Debug)]
pub enum InteractError<E> {
    /// Provided callback has panicked.
    Panic(Box<dyn Any + Send + 'static>),

    /// Callback was aborted.
    Aborted,

    /// Backend returned an error.
    Backend(E),
}

impl<E: fmt::Display> fmt::Display for InteractError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic(_) => write!(f, "Panic"),
            Self::Aborted => write!(f, "Aborted"),
            Self::Backend(e) => write!(f, "Backend error: {}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for InteractError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Panic(_) | Self::Aborted => None,
            Self::Backend(e) => Some(e),
        }
    }
}

/// Wrapper for objects which only provides blocking functions that need to be
/// called on a separate thread.
///
/// Access to the wrapped object is provided via the [`SyncWrapper::interact()`]
/// method.
#[must_use]
pub struct SyncWrapper<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    obj: Arc<Mutex<Option<T>>>,
    runtime: Runtime,
    _error: PhantomData<fn() -> E>,
}

// Implemented manually to avoid unnecessary trait bound on `E` type parameter.
impl<T, E> fmt::Debug for SyncWrapper<T, E>
where
    T: fmt::Debug + Send + 'static,
    E: Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SyncWrapper")
            .field("obj", &self.obj)
            .field("runtime", &self.runtime)
            .field("_error", &self._error)
            .finish()
    }
}

impl<T, E> SyncWrapper<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    /// Creates a new wrapped object.
    pub async fn new<F>(runtime: Runtime, f: F) -> Result<Self, E>
    where
        F: FnOnce() -> Result<T, E> + Send + 'static,
    {
        let result = match runtime.spawn_blocking(move || f()).await {
            // FIXME: Panicking when the creation panics is not nice.
            // In order to handle this properly the Manager::create
            // methods needs to support a custom error enum which
            // supports a Panic variant.
            Err(SpawnBlockingError::Panic(e)) => panic!("{:?}", e),
            Ok(obj) => obj,
        };
        result.map(|obj| Self {
            obj: Arc::new(Mutex::new(Some(obj))),
            runtime,
            _error: PhantomData::default(),
        })
    }

    /// Interacts with the underlying object.
    ///
    /// Expects a closure that takes the object as its parameter.
    /// The closure is executed in a separate thread so that the async runtime
    /// is not blocked.
    pub async fn interact<F, R>(&self, f: F) -> Result<R, InteractError<E>>
    where
        F: FnOnce(&mut T) -> Result<R, E> + Send + 'static,
        R: Send + 'static,
    {
        let arc = self.obj.clone();
        self.runtime
            .spawn_blocking(move || {
                let mut guard = arc.lock().unwrap();
                let conn = guard.as_mut().unwrap();
                f(conn)
            })
            .await
            .map_err(|e| match e {
                SpawnBlockingError::Panic(p) => InteractError::Panic(p),
            })?
            .map_err(InteractError::Backend)
    }

    /// Indicates whether the underlying [`Mutex`] has been poisoned.
    ///
    /// This happens when a panic occurs while interacting with the object.
    pub fn is_mutex_poisoned(&self) -> bool {
        self.obj.is_poisoned()
    }

    /// Lock the underlying mutex and return a guard for the inner
    /// object.
    pub fn lock(&self) -> Result<SyncGuard<'_, T>, PoisonError<MutexGuard<'_, Option<T>>>> {
        self.obj.lock().map(SyncGuard)
    }

    /// Try to lock the underlying mutex and return a guard for the
    /// inner object.
    pub fn try_lock(&self) -> Result<SyncGuard<'_, T>, TryLockError<MutexGuard<'_, Option<T>>>> {
        self.obj.try_lock().map(SyncGuard)
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

/// This guard is returned when calling `SyncWrapper::lock` or
/// `SyncWrapper::try_lock`. This is basicly just a wrapper around
/// a `MutexGuard` but hides some implementation details.
///
/// **Important:** Any blocking operation using this object
/// should be executed on a separate thread (e.g. via `spawn_blocking`).
#[derive(Debug)]
pub struct SyncGuard<'a, T: Send>(MutexGuard<'a, Option<T>>);

impl<'a, T: Send> Deref for SyncGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<'a, T: Send> DerefMut for SyncGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<'a, T: Send> AsRef<T> for SyncGuard<'a, T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref().unwrap()
    }
}

impl<'a, T: Send> AsMut<T> for SyncGuard<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.0.as_mut().unwrap()
    }
}
