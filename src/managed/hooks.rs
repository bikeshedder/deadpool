//! Hooks allowing to run code when creating and/or recycling objects.

use std::{fmt, future::Future, pin::Pin};

use super::{
    metrics::{Metrics, WithMetrics},
    PoolError,
};

/// The result returned by hooks
pub type HookResult<E> = Result<(), HookError<E>>;

/// The boxed future that should be returned by async hooks
pub type HookFuture<'a, E> = Pin<Box<dyn Future<Output = HookResult<E>> + Send + 'a>>;

/// Function signature for sync callbacks
pub type SyncFn<T, E> = dyn Fn(&mut T, &Metrics) -> HookResult<E> + Sync + Send;

/// Function siganture for async callbacks
pub type AsyncFn<T, E> = dyn for<'a> Fn(&'a mut T, &'a Metrics) -> HookFuture<'a, E> + Sync + Send;

/// Wrapper for hook functions
pub enum Hook<T, E> {
    /// Use a plain function (non-async) as a hook
    Fn(Box<SyncFn<T, E>>),
    /// Use an async function as a hook
    AsyncFn(Box<AsyncFn<T, E>>),
}

impl<T, E> Hook<T, E> {
    /// Create Hook from sync function
    pub fn sync_fn(f: impl Fn(&mut T, &Metrics) -> HookResult<E> + Sync + Send + 'static) -> Self {
        Self::Fn(Box::new(f))
    }
    /// Create Hook from async function
    pub fn async_fn(
        f: impl for<'a> Fn(&'a mut T, &'a Metrics) -> HookFuture<'a, E> + Sync + Send + 'static,
    ) -> Self {
        Self::AsyncFn(Box::new(f))
    }
}

impl<T, E> fmt::Debug for Hook<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fn(_) => f
                .debug_tuple("Fn")
                //.field(arg0)
                .finish(),
            Self::AsyncFn(_) => f
                .debug_tuple("AsyncFn")
                //.field(arg0)
                .finish(),
        }
    }
}

/// Error structure which which can abort the creation and recycling
/// of objects.
///
/// There are two variants [`HookError::Continue`] tells the pool
/// to continue the running [`Pool`] operation ([`get`],
/// [`timeout_get`] or [`try_get`]) while [`HookError::Abort`] does abort
/// that operation with an error.
///
/// [`Pool`]: crate::managed::Pool
/// [`get`]: crate::managed::Pool::get
/// [`timeout_get`]: crate::managed::Pool::timeout_get
/// [`try_get`]: crate::managed::Pool::try_get
#[derive(Debug)]
pub enum HookError<E> {
    /// This variant can be returned by hooks if the object should be
    /// discarded but the operation should be continued.
    Continue(Option<HookErrorCause<E>>),
    /// This variant causes the object to be discarded and aborts the
    /// operation.
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

pub(crate) struct HookVec<T, E> {
    vec: Vec<Hook<T, E>>,
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<T, E> fmt::Debug for HookVec<T, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Hooks")
            // FIXME
            //.field("fns", &self.fns)
            .finish()
    }
}

// Implemented manually to avoid unnecessary trait bound on `M` type parameter.
impl<T, E> Default for HookVec<T, E> {
    fn default() -> Self {
        Self { vec: Vec::new() }
    }
}

impl<T, E> HookVec<T, E> {
    pub(crate) async fn apply(
        &self,
        obj: &mut WithMetrics<T>,
        error: fn(e: HookError<E>) -> PoolError<E>,
    ) -> Result<Option<HookError<E>>, PoolError<E>> {
        for hook in &self.vec {
            let result = match hook {
                Hook::Fn(f) => f(&mut obj.obj, &obj.metrics),
                Hook::AsyncFn(f) => f(&mut obj.obj, &obj.metrics).await,
            };
            match result {
                Ok(()) => {}
                Err(e) => match e {
                    HookError::Continue(_) => return Ok(Some(e)),
                    HookError::Abort(_) => return Err(error(e)),
                },
            }
        }
        Ok(None)
    }
    pub(crate) fn push(&mut self, hook: impl Into<Hook<T, E>>) {
        self.vec.push(hook.into());
    }
}

/// Collection of all the hooks that can be configured for a [`Pool`].
///
/// [`Pool`]: super::Pool
pub struct Hooks<T, E> {
    pub(crate) post_create: HookVec<T, E>,
    pub(crate) pre_recycle: HookVec<T, E>,
    pub(crate) post_recycle: HookVec<T, E>,
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
            pre_recycle: HookVec::default(),
            post_create: HookVec::default(),
            post_recycle: HookVec::default(),
        }
    }
}
