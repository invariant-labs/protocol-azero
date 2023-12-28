#!/bin/bash

npm run node:start &
npm run test &
test_pid=$!

wait $test_pid
npm run node:stop