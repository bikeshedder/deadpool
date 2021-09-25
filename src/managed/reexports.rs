//! This module contains all things that should be reexported
//! by backend implementations in order to avoid direct dependencies
//! on the `deadpool` crate itself.
//!
//! Crates based on [`deadpool::managed`] should include this line:
//! ```rust
//! pub use deadpool::managed::reexports::*;
//! ```

pub use crate::{
    managed::{PoolConfig, Timeouts},
    Runtime,
};
