# Change Log

## v0.2.0 (unreleased)

* __Breaking:__ Replace `config` feature with `serde` (opted out by default)
* Fix `std::error::Error::source` implementations for library errors
* Remove unused `tokio` dependency
* Async unaware diesel connections are now wrapped inside
  a `deadpool::managed::sync::SyncWrapper` which ensures that
  all database operations are run in a separate threads.

## v0.1.2

* Remove `unsafe impl` by better usage of `PhantomData`

## v0.1.1

* Fix title and crates.io badge in README

## v0.1.0

* First release
