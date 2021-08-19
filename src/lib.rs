#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(
    nonstandard_style,
    rust_2018_idioms,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links
)]
#![forbid(non_ascii_idents, unsafe_code)]
#![warn(
    deprecated_in_future,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    unreachable_pub,
    unused_import_braces,
    unused_labels,
    unused_lifetimes,
    unused_qualifications,
    unused_results
)]

#[cfg(feature = "managed")]
#[cfg_attr(docsrs, doc(cfg(feature = "managed")))]
pub mod managed;

#[cfg(feature = "unmanaged")]
#[cfg_attr(docsrs, doc(cfg(feature = "unmanaged")))]
pub mod unmanaged;

mod runtime;

pub use self::runtime::Runtime;

/// The current pool status.
#[derive(Clone, Copy, Debug)]
pub struct Status {
    /// The maximum size of the pool.
    pub max_size: usize,

    /// The current size of the pool.
    pub size: usize,

    /// The number of available objects in the pool. If there are no
    /// objects in the pool this number can become negative and stores the
    /// number of futures waiting for an object.
    pub available: isize,
}
