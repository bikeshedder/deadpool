# Deadpool runtime abstraction [![Latest Version](https://img.shields.io/crates/v/deadpool-runtime.svg)](https://crates.io/crates/deadpool-runtime) ![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg "Unsafe forbidden") [![Rust 1.54+](https://img.shields.io/badge/rustc-1.54+-lightgray.svg "Rust 1.54+")](https://blog.rust-lang.org/2021/07/29/Rust-1.54.0.html)

Deadpool is a dead simple async pool for connections and objects
of any type.

This crate provides a simple `Runtime` enum that can be used to
target multiple runtimes. This crate avoids boxed futures and
and only implements things actually needed by the `deadpool` crates.

**Note:** This crate is intended for making the development of
`deadpool-*` crates easier. Other libraries and binary projects
normally should not use this directly and use some provided
reexports by the crates using it.
