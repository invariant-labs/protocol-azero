#!/usr/bin/env bash
SEED="//Alice"
URL="wss://ws.test.azero.dev"
CONTRACT="5DNQ2JCwGip6iUuHgam2Y7FZHbJfJsr8M9AQCdVzkBV9LMSM"

ABI_PATH=$(pwd)/target/ink/contract.json
echo "ABI path: ${ABI_PATH}"

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message new \
        --args 0 \
        --output-json \
        --skip-confirm \
        --execute
        
# cargo contract call ABI_PATH --suri "$SEED" --url "$URL" \
#         --contract 5DNQ2JCwGip6iUuHgam2Y7FZHbJfJsr8M9AQCdVzkBV9LMSM \
#         --message get_protocol_fee \
        