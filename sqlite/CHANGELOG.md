## v0.3.0 (unreleased)

* __Breaking:__ Replace `deadpool::managed::sync` by
  `deadpool-sync::SyncWrapper` which fixes the return type
  of the `interact` method.

## v0.2.0

* __Breaking:__ Replace `config` feature with `serde` (opted out by default)
* Add support for multiple runtimes
* Fix panic handling inside the `interact` method
* Wrap blocking drop method of connections inside `spawn_blocking`
* Remove unused `futures`, `tokio` and `log` dependencies

## v0.1.0

* First release
