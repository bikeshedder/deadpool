# Change Log

## v0.2.1

* `Client` no longer implements `DerefMut` which was not needed anyways.
* `Client.transaction` now returns a wrapped transaction object which
    utilizes the statement cache of the wrapped client.

## v0.2.0

* First release
