#!/bin/bash

set -e

cd ..
cd sdk
./build.sh
npm run node:start &
sleep 5
cd ..
cd scripts
npm i
npm run start &
test_pid=$!

wait $test_pid
test_status=$?

exit $test_status