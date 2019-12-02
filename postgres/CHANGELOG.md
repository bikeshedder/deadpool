# Change Log

## v0.2.3

* Add documentation for `docs.rs`
* Improve example in `README.md` and crate root

## v0.2.2

* Update to `tokio 0.2` and `tokio-postgres 0.5.0-alpha.2`

## v0.2.1

* `deadpool_postgres::Client` no longer implements `DerefMut` which was not
    needed anyways.
* `deadpool_postgres::Client.transaction` now returns a wrapped transaction
    object which utilizes the statement cache of the wrapped client.

## v0.2.0

* First release
