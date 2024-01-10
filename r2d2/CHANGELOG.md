# Change Log

## unreleased

* Update `deadpool` dependency to version `0.11`
* Remove `async_trait` dependency
* Bump up MSRV to `1.75`

## v0.3.0

* Update `deadpool` dependency to version `0.10`
* Add `tracing` feature
* Bump up MSRV to `1.63` to match the one of `tokio`

## v0.2.0

* __Breaking:__ Replace `deadpool::managed::sync` by
  `deadpool-sync::SyncWrapper` which fixes the return type
  of the `interact` method.

## v0.1.0

* First release
