//! Deadpool is a dead simple async pool for connections and objects
//! of any type.
//!
//! It provides two implementations:
//! - A `managed` one which requires a `Manager` trait which is responsible
//!   for creating and recycling objects as they are needed.
//! - An `unmanaged` one which requires the objects to be created upfront and
//!   is a lot simpler, too.
#![warn(missing_docs)]

#[cfg(feature = "managed")]
mod compat_0_4;
#[cfg(feature = "managed")]
pub mod managed;
#[cfg(feature = "managed")]
pub use compat_0_4::*;

#[cfg(feature = "unmanaged")]
pub mod unmanaged;

#[derive(Debug)]
/// The current pool status.
pub struct Status {
    /// The size of the pool
    pub size: usize,
    /// The number of available objects in the pool. If there are no
    /// objects in the pool this number can become negative and stores the
    /// number of futures waiting for an object.
    pub available: isize,
}
