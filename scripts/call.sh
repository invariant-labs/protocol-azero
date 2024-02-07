#!/usr/bin/env bash
SEED="//Alice"
URL="wss://ws.test.azero.dev"
CONTRACT="5DJZgrfCnvQ9LY7UqbGJnXabTjCeY8xM6g77BiP4aof7vjqC"

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message InvariantTrait::get_protocol_fee \
        --execute \
        --skip-confirm