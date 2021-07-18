//! This module contains hooks which allow to run code when
//! creating and/or recycling objects.
use std::fmt;

use super::Manager;

/// This errors can be returned by hooks which will abort the
/// creation and/or recycling of objects.
#[derive(Debug)]
pub enum HookError<E> {
    /// The hook failed for some other reason
    Message(String),
    /// The error was caused by the backend
    Backend(E),
}

impl<E> fmt::Display for HookError<E>
where
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(msg) => write!(f, "{}", msg),
            Self::Backend(e) => write!(f, "{}", e),
        }
    }
}

impl<E> std::error::Error for HookError<E> where E: std::error::Error {}

/// Trait for `post_create` hooks.
#[async_trait::async_trait]
pub trait PostCreate<M: Manager>: Sync + Send {
    /// The hook method which is called after creating a new object
    async fn post_create(&self, obj: &mut M::Type) -> Result<(), HookError<M::Error>>;
}

/// Trait for `post_recycle` hooks.
#[async_trait::async_trait]
pub trait PostRecycle<M: Manager>: Sync + Send {
    /// The hook method which is called after recycling an existing object
    async fn post_recycle(&self, obj: &mut M::Type) -> Result<(), HookError<M::Error>>;
}

/// This struct contains all the hooks that can be configured
/// for a pool.
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
