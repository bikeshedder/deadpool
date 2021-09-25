//! Hooks allowing to run code when creating and/or recycling objects.

use std::fmt;

use async_trait::async_trait;

use super::{
    metrics::{Metrics, WithMetrics},
    PoolError,
};

/// Error structure which which can abort the creation and recycling
/// of objects.
#[derive(Debug)]
pub enum HookError<E> {
    /// This field is used by the pool to know wether to abort the entire
    /// operation (typically `Pool::get` or `Pool::get_timeout`) should
    /// be aborted.
    Continue(Option<HookErrorCause<E>>),
    /// This is the optional cause of the error as hooks might just want to
    /// discard an object (abort=false) and not abort.
    Abort(HookErrorCause<E>),
}

/// Possible errors returned by [`Hooks`]
#[derive(Debug)]
pub enum HookErrorCause<E> {
    /// Hook failed for some other reason.
    Message(String),

    /// Hook failed for some other reason.
    StaticMessage(&'static str),

    /// Error caused by the backend.
    Backend(E),
}

impl<E> HookError<E> {
    /// Get optional cause of this error
    pub fn cause(&self) -> Option<&HookErrorCause<E>> {
        match self {
            Self::Continue(option) => option.as_ref(),
            Self::Abort(cause) => Some(cause),
        }
    }
}

impl<E: fmt::Display> fmt::Display for HookError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cause() {
            Some(HookErrorCause::Message(msg)) => write!(f, "{}", msg),
            Some(HookErrorCause::StaticMessage(msg)) => write!(f, "{}", msg),
            Some(HookErrorCause::Backend(e)) => write!(f, "{}", e),
            None => write!(f, "No cause given"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for HookError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self.cause() {
            Some(HookErrorCause::Message(_)) => None,
            Some(HookErrorCause::StaticMessage(_)) => None,
            Some(HookErrorCause::Backend(e)) => Some(e),
            None => None,
        }
    }
}

/// Abstraction of `post_recycle` hooks.
#[async_trait]
pub trait HookCallback<T, E>: Sync + Send {
    /// The hook method which is called after recycling an existing [`Object`].
    ///
    /// [`Object`]: super::Object
    async fn call(&self, obj: &mut T, metrics: &Metrics) -> Result<(), HookError<E>>;
}

impl<T, E> fmt::Debug for dyn HookCallback<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

pub(crate) struct HookCallbacks<T, E> {
    callbacks: Vec<Box<dyn HookCallback<T, E>>>,
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<T, E> fmt::Debug for HookCallbacks<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hooks")
            .field("callbacks", &self.callbacks)
            .finish()
    }
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<T, E> Default for HookCallbacks<T, E> {
    fn default() -> Self {
        Self {
            callbacks: Vec::new(),
        }
    }
}

impl<T, E> HookCallbacks<T, E> {
    pub(crate) async fn apply(
        &self,
        obj: &mut WithMetrics<T>,
        error: fn(e: HookError<E>) -> PoolError<E>,
    ) -> Result<Option<HookError<E>>, PoolError<E>> {
        for hook in &self.callbacks {
            match hook.call(&mut obj.obj, &obj.metrics).await {
                Ok(()) => {}
                Err(e) => match e {
                    HookError::Continue(_) => return Ok(Some(e)),
                    HookError::Abort(_) => return Err(error(e)),
                },
            }
        }
        Ok(None)
    }
    pub(crate) fn push(&mut self, callback: impl HookCallback<T, E> + 'static) {
        self.callbacks.push(Box::new(callback));
    }
}

/// Collection of all the hooks that can be configured for a [`Pool`].
///
/// [`Pool`]: super::Pool
pub struct Hooks<T, E> {
    pub(crate) post_create: HookCallbacks<T, E>,
    pub(crate) pre_recycle: HookCallbacks<T, E>,
    pub(crate) post_recycle: HookCallbacks<T, E>,
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<T, E> fmt::Debug for Hooks<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hooks")
            .field("post_create", &self.post_create)
            .field("pre_recycle", &self.post_recycle)
            .field("post_recycle", &self.post_recycle)
            .finish()
    }
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<T, E> Default for Hooks<T, E> {
    fn default() -> Self {
        Self {
            pre_recycle: HookCallbacks::default(),
            post_create: HookCallbacks::default(),
            post_recycle: HookCallbacks::default(),
        }
    }
}
