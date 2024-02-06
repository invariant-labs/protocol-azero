#!/bin/bash

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
  echo "Cargo.toml not found!"
  exit 1
fi

# Find the line number where [lib] is located
lib_line_number=$(grep -n "\[lib\]" Cargo.toml | cut -d: -f1)

# Check if [lib] is found
if [ -z "$lib_line_number" ]; then
  echo "[lib] not found in Cargo.toml!"
  exit 1
fi

# Add the crate-type line after [lib]
sed -i "${lib_line_number}a crate-type = [\"cdylib\",\"rlib\"]" Cargo.toml

# Modify default value in Cargo.toml
sed -i 's/default = \["std"\]/default = \[\]/' Cargo.toml


# Run wasm-pack build command
wasm-pack build --out-name index --out-dir ../../sdk/clamm --features wasm 

# Remove the crate-type line
sed -i "/crate-type = \[\"cdylib\",\"rlib\"\]/d" Cargo.toml

# Restore default value in Cargo.toml
sed -i 's/default = \[\]/default = \["std"\]/' Cargo.toml

# Change directory to /clamm
cd ../../sdk/clamm

# Run npm pkg set type='module'
npm pkg set type='module'
