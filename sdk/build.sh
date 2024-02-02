#!/bin/bash

npm i
npm run lint
npm run docs:copy
npm run wasm:build
npm run wasm:package
npm run build
build_status=$?
exit $build_status