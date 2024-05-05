#!/bin/bash

# Usage: ./publish <version>
# For example: ./publish 0.1.0

jq '.name = "@invariant-labs/a0-sdk-wasm"' src/wasm/pkg/package.json > temp.json && mv temp.json src/wasm/pkg/package.json

if [ -z "$1" ]; then
    echo "Please provide the version to publish."
    exit 1
fi

jq ".version = \"$1\"" src/wasm/pkg/package.json > temp.json && mv temp.json src/wasm/pkg/package.json

cd src/wasm/pkg
npm publish