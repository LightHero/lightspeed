#!/usr/bin/env sh
set -e
set -x
export RUST_BACKTRACE=full

cargo test 
cargo test --features axum
cargo test --features poem
cargo test --features poem_openapi
cargo test --all-features
