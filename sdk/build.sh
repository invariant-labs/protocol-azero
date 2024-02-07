#!/bin/bash

npm i
npm run lint
npm run docs:copy
npm run wasm:build
npm run build
build_status=$?
exit $build_status