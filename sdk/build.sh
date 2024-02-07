#!/bin/bash

npm link
npm i
npm run lint
npm link
npm i
npm run docs:copy
npm link
npm i
npm run wasm:build
npm link
npm i
npm run wasm:package
npm link
npm i
npm run build
build_status=$?
exit $build_status