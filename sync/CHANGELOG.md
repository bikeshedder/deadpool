## v0.1.2 (unreleased)

* Add `tracing` feature to `README`
* Fix MSRV (this was the reason for yanking the `0.1.1` release) and bump it up to `1.63` to match the one of `tokio`

## v0.1.1 (yanked)

* Replace `deadpool` dependency by `deadpool-runtime`. This is a
  non-breaking change as the `Runtime` and `SpawnBlockingError`
  types were a re-export from `deadpool-runtime` anyways.

## v0.1.0

* First release
