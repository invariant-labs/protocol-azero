#!/bin/bash
set -e

cd src

# Build and test math module
# cd math
# cargo build
# cargo test

# Build trackable result
cd traceable_result
cargo build

cd ..
# Build decimal
cd decimal
cargo build

cd ../..
# Build and test parent module

cargo test --features e2e-tests

# build contract
cargo contract build