# Deadpool for synchroneous code [![Latest Version](https://img.shields.io/crates/v/deadpool-sync.svg)](https://crates.io/crates/deadpool-sync) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.54+](https://img.shields.io/badge/rustc-1.54+-lightgray.svg "Rust 1.54+")](https://blog.rust-lang.org/2021/07/29/Rust-1.54.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crates provides helpers for writing pools for objects that don't
support async and need to be run inside a thread.

**Note:** This crate is intended for making the development of
`deadpool-*` crates easier. Other libraries and binary projects
normally should not use this directly and use some provided
reexports by the crates using it.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0)>
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT)>

at your option.
