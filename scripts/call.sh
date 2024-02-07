#!/usr/bin/env bash
SEED="//Alice"
URL="wss://ws.test.azero.dev"
CONTRACT="5DNQ2JCwGip6iUuHgam2Y7FZHbJfJsr8M9AQCdVzkBV9LMSM"

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message InvariantTrait::get_protocol_fee \
        --execute \
        --skip-confirm