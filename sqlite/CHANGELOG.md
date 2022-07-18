## v0.5.0 (unreleased)

- Update `rusqlite` dependency to version `0.28`

## v0.4.0

- Update `rusqlite` dependency to version `0.27`

## v0.3.1

* Config structs now implement `Serialize`

## v0.3.0

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
