#!/bin/bash

npm run node:start &
npm run test &
test_pid=$!

wait $test_pid
test_status=$?

npm run node:stop
exit $test_status