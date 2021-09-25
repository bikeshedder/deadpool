//! This module contains all things that should be reexported
//! by backend implementations in order to avoid direct
//! dependencies on the `deadpool` crate itself.
//!
//! This module contains the variant of this module used
//! by *sync* backends.
//!
//! Crates based on [`deadpool::managed`] should include this line:
//! ```rust
//! pub use deadpool::managed::reexports::*;`
//! ```

pub use super::super::reexports::*;
pub use super::{InteractError, SyncGuard};
