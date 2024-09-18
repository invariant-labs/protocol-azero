#!/bin/bash
set -e

cd src

# Test token module
cd token
cargo test
cd ..

# Test trackable result
cd traceable_result
cargo test
cd ../..

# Test decimal
cargo test -p decimal

cargo fmt --all -- --check
cargo clippy --all-targets -- --no-deps -D warnings

cargo test --features e2e-tests

# Build contract
cargo contract build
cargo contract build --release

# Build and test SDK
cd sdk
chmod +x tests.sh build.sh package.sh
./build.sh
./tests.sh