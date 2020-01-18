#!/bin/bash

set -xe

cargo build
cargo build --no-default-features
cargo build --no-default-features --features managed
cargo build --no-default-features --features unmanaged
cargo build --no-default-features --features config
cargo build --no-default-features --features managed,config
cargo build --no-default-features --features unmanaged,config
cargo build --no-default-features --features managed,unmanaged

(
	cd postgres
	cargo build
	cargo build --no-default-features
)

(
	cd redis
	cargo build
	cargo build --no-default-features
)

(
	cd lapin
	cargo build
	cargo build --no-default-features
)

