#!/bin/bash

# Parameters
chain_id="$1"
node="$2"
gas_price="$3"
signer_key_acc_address="$4"
migrate_msg="$5" 
resolver_contract_address="$6"
contract_name="$7"
contracts_dir="$8"
wallet_password="$9"

# Function to store contract code and get the code ID
store_contract() {
    local contract_name=$1
    local contract_file="${contracts_dir}/${contract_name}-aarch64.wasm"
    echo >&2 "Storing $contract_name..."
    STORE_OUTPUT=$(yes $wallet_password | archwayd tx wasm store "$contract_file" \
        --from="$signer_key_acc_address" \
        --chain-id="$chain_id" \
        --gas-prices="$gas_price" --gas=10000000 \
        --node="$node" \
        --output json --yes)

    # Extract txhash from STORE_OUTPUT
    TXHASH=$(echo "$STORE_OUTPUT" | jq -r '.txhash')
    echo >&2 "Transaction Hash: $TXHASH"

    sleep 10

    # Query transaction by txhash to get detailed info, including possibly the code ID
    QUERY_OUTPUT=$(archwayd query tx --type=hash "$TXHASH" --node="$node" --chain-id="$chain_id" --output json)
    CODE_ID=$(echo "$QUERY_OUTPUT" | jq -r '.logs[0].events[] | select(.type == "cosmwasm.wasm.v1.EventCodeStored").attributes[] | select(.key == "code_id").value | gsub("\"";"")')

    echo >&2 "CODE_ID = $CODE_ID"
    echo >&2 "Stored $contract_name with code ID: $CODE_ID"
    echo $CODE_ID
}

# Function to migrate a contract
migrate_contract() {
    local contract_address=$1
    local new_code_id=$2
    local migrate_msg=$3
    echo >&2 "Migrating contract at address $contract_address to new code ID: $new_code_id..."
    MIGRATE_OUTPUT=$(yes "$wallet_password" | archwayd tx wasm migrate "$contract_address" "$new_code_id" "$migrate_msg" \
        --from="$signer_key_acc_address" \
        --chain-id="$chain_id" \
        --node="$node" \
        --gas-prices="$gas_price" --gas=10000000 \
        --output json --yes)

    # Extract txhash from MIGRATE_OUTPUT
    TXHASH=$(echo "$MIGRATE_OUTPUT" | jq -r '.txhash')
    echo >&2 "Migration Transaction Hash: $TXHASH"

    sleep 10 # Wait for the transaction to be processed

    # Query transaction by txhash to get detailed info, including success or failure
    QUERY_OUTPUT=$(archwayd query tx --type=hash "$TXHASH" --node="$node" --chain-id="$chain_id" --output json)
    echo >&2 "Migration tx: $QUERY_OUTPUT"
}

# Store the new contract code and retrieve the new code ID
new_code_id=$(store_contract "$contract_name")

# Execute migration for the specified contract using the new code ID
migrate_contract "$resolver_contract_address" "$new_code_id" "$migrate_msg"

echo "Migration of $contract_name complete with new code ID: $new_code_id."
