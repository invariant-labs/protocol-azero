#!/usr/bin/env bash
SEED="//Alice"
URL="wss://ws.test.azero.dev"
CONTRACT="5DNQ2JCwGip6iUuHgam2Y7FZHbJfJsr8M9AQCdVzkBV9LMSM"

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message Invariant::get_protocol_fee \
        --execute \
        --skip-confirm
        
        
# cargo contract call ABI_PATH --suri "$SEED" --url "$URL" \
#         --contract 5DNQ2JCwGip6iUuHgam2Y7FZHbJfJsr8M9AQCdVzkBV9LMSM \
#         --message get_protocol_fee \
        