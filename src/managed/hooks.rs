//! Hooks allowing to run code when creating and/or recycling objects.

use std::fmt;

use async_trait::async_trait;

use super::{metrics::Metrics, Manager};

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

/// Abstraction of `post_create` hooks.
#[async_trait]
pub trait PostCreate<M: Manager>: Sync + Send {
    /// The hook method which is called after creating a new [`Object`].
    ///
    /// [`Object`]: super::Object
    async fn post_create(&self, obj: &mut M::Type) -> Result<(), HookError<M::Error>>;
}

impl<M: Manager> fmt::Debug for dyn PostCreate<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

/// Abstraction of `post_recycle` hooks.
#[async_trait]
pub trait PostRecycle<M: Manager>: Sync + Send {
    /// The hook method which is called after recycling an existing [`Object`].
    ///
    /// [`Object`]: super::Object
    async fn post_recycle(
        &self,
        obj: &mut M::Type,
        metrics: &Metrics,
    ) -> Result<(), HookError<M::Error>>;
}

impl<M: Manager> fmt::Debug for dyn PostRecycle<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self)
    }
}

/// Collection of all the hooks that can be configured for a [`Pool`].
///
/// [`Pool`]: super::Pool
pub struct Hooks<M: Manager> {
    pub(crate) post_create: Vec<Box<dyn PostCreate<M>>>,
    pub(crate) post_recycle: Vec<Box<dyn PostRecycle<M>>>,
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<M: Manager> fmt::Debug for Hooks<M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hooks")
            .field("post_create", &self.post_create)
            .field("post_recycle", &self.post_recycle)
            .finish()
    }
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<M: Manager> Default for Hooks<M> {
    fn default() -> Self {
        Self {
            post_create: Vec::new(),
            post_recycle: Vec::new(),
        }
    }
}
