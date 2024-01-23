#!/bin/bash

npm run node:start &
npm run test &
test_pid=$!

wait $test_pid
ps aux | grep substrate-contracts-node | tr -s ' ' | cut -d ' ' -f 2 | head -n 1 | xargs -r kill -9