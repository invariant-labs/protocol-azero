#!/usr/bin/env bash

# This script does the following:
# * deploys Highlighted Posts and Bulletin Board contracts
# * instantiates them
# * stores addreses in the `addresses.json` file in the current directory
#
# What it does not do:
# * it doesn't build the contracts - assumes they're already built

set -euo pipefail

# Quiet versions of pushd and popd
pushd () {
    command pushd "$@" > /dev/null
}

popd () {
    command popd "$@" > /dev/null
}

CONTRACTS_PATH=$(pwd)
echo "Path ${CONTRACTS_PATH}"

NODE_URL=${NODE_URL:="ws://localhost:9944"}
AUTHORITY_SEED=${AUTHORITY_SEED:="//Alice"}

echo "node=${NODE_URL}"
echo "authority_seed=${AUTHORITY_SEED}"

function upload_contract {

    local  __resultvar=$1
    local contract_name=$2

    echo "contract_name ${contract_name}"


    pushd "$CONTRACTS_PATH"/$contract_name

    echo "Uploading ${contract_name}"

    # --- UPLOAD CONTRACT CODE

    code_hash=$(cargo contract upload --quiet --url "$NODE_URL" --suri "$AUTHORITY_SEED" --execute --skip-confirm)
    code_hash=$(echo "${code_hash}" | grep hash | tail -1 | cut -c 14-)

    eval $__resultvar=${code_hash}

    popd
}

function extract_contract_addresses {
    jq  '.events[] | select((.pallet == "Contracts") and (.name = "Instantiated")) | .fields[] | select(.name == "contract") | .value.Literal'
}

function extract_from_quotes {
    echo $1 | tr -d '"'
}

upload_contract INVARIANT_CODE_HASH ./
echo "Bulletin Board code hash: ${INVARIANT_CODE_HASH}"

# --- instantiate contract

pushd ${CONTRACTS_PATH}

# Using temporary file as piping JSON from env variable crates problems with escaping.
temp_file=$(mktemp)
# Remove temporary file when finished.
trap "rm -f $temp_file" 0 2 3 15 

SALT=${INVARIANT_VERSION:-12389012}
INVARIANT_CONTRACT_FILE="target/ink/contract.contract"

echo "Instantiating Bulletin Board contract (version: ${SALT})"
# cargo contract instantiate --url "$NODE_URL" --salt ${SALT} --suri "$AUTHORITY_SEED" $INVARIANT_CONTRACT_FILE --constructor new --args "0" --execute --skip-confirm --output-json > temp_file

# No salt instantiation
cargo contract instantiate --url "$NODE_URL" --suri "$AUTHORITY_SEED" $INVARIANT_CONTRACT_FILE --constructor new --args "0" --execute --skip-confirm --output-json > temp_file

INVARIANT_ADDRESS=$(cat temp_file | jq  '.events[] | select((.pallet == "Contracts") and (.name = "Instantiated")) | .fields[] | select(.name == "contract") | .value.Literal' | tail -1 | tr -d '"')
if [[ -z BULLETIN_BOARD_ADDRESS && -v BULLETIN_BOARD_ADDRESS ]]; then
    echo "Empty BULLETIN_BOARD_ADDRESS"
    exit 1
fi

echo "Invariant instance address: ${INVARIANT_ADDRESS}"

popd

jq -n --arg INVARIANT_CODE_HASH "$INVARIANT_CODE_HASH" \
    --arg INVARIANT_ADDRESS "$INVARIANT_ADDRESS" \
    '{
        INVARIANT_CODE_HASH: $INVARIANT_CODE_HASH,
        INVARIANT_ADDRESS: $INVARIANT_ADDRESS        
    }' > ${PWD}/scripts/addresses.json

echo "Contract addresses stored in addresses.json"
exit 0