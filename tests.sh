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
cd ..


# Test decimal
cd decimal
cargo test
cd decimal_core
cd ../../..


cargo test --features e2e-tests

# build contract
cargo contract build