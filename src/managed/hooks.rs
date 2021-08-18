//! Hooks allowing to run code when creating and/or recycling objects.

use std::fmt;

use async_trait::async_trait;

use super::Manager;

/// Possible errors returned by [`Hooks`] which will abort the creation and/or
/// recycling of objects.
#[derive(Debug)]
pub enum HookError<E> {
    /// Hook failed for some other reason.
    Message(String),

    /// Error caused by the backend.
    Backend(E),
}

impl<E: fmt::Display> fmt::Display for HookError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{}", msg),
            Self::Backend(e) => write!(f, "{}", e),
        }
    }
}

impl<E: std::error::Error> std::error::Error for HookError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Message(_) => None,
            Self::Backend(e) => Some(e),
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

/// Abstraction of `post_recycle` hooks.
#[async_trait]
pub trait PostRecycle<M: Manager>: Sync + Send {
    /// The hook method which is called after recycling an existing [`Object`].
    ///
    /// [`Object`]: super::Object
    async fn post_recycle(&self, obj: &mut M::Type) -> Result<(), HookError<M::Error>>;
}

/// Collection of all the hooks that can be configured for a [`Pool`].
///
/// [`Pool`]: super::Pool
pub struct Hooks<M: Manager> {
    pub(crate) post_create: Vec<Box<dyn PostCreate<M>>>,
    pub(crate) post_recycle: Vec<Box<dyn PostRecycle<M>>>,
}

impl<M: Manager> Default for Hooks<M> {
    fn default() -> Self {
        Self {
            post_create: Vec::new(),
            post_recycle: Vec::new(),
        }
    }
}
