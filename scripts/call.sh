#!/usr/bin/env bash
SEED="//Alice"
URL="wss://ws.test.azero.dev"
CONTRACT="5Gz1A9SSozUrf49unQT5qW6yNUvCxBLi2i16UqMUQC2a9uVg"

# tokenX = 5DJZgrfCnvQ9LY7UqbGJnXabTjCeY8xM6g77BiP4aof7vjqC
# tokenY = 5ExiLppPgWKJDfbyS1jR7oMBHzPQB7PKSrXx3aN5JaxiJKV4

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message InvariantTrait::get_protocol_fee \
        --execute \
        --skip-confirm

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message InvariantTrait::prepare_contract \
        --skip-dry-run \
        --gas 259058343000 \
        --proof-size 1000000000000 \
        --execute \
        --skip-confirm

cargo contract call --suri "$SEED" --url "$URL" \
        --contract $CONTRACT \
        --message InvariantTrait::get_tickmap \
        --skip-dry-run \
        --gas 259058343000 \
        --proof-size 1000000000000 \
        --execute \
        --skip-confirm