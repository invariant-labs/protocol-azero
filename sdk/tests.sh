#!/bin/bash

npm run node:start &
npm run test:get-liquidity-ticks
test_status=$?

npm run node:stop
exit $test_status