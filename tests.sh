#!/bin/bash
set -e

cd src

# Build and test math module
cd math
cargo build
cargo test

# Build and test parent module
cd ..
cargo build
cargo test