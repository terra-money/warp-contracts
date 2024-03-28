#!/bin/bash

# Parameters
chain_id="$1"
node="$2"
gas_price="$3"
signer_key_acc_address="$4"
instantiate_templates_msg_template="$5"
instantiate_controller_msg_template="$6"
contracts_dir="$7" # Path to the contracts directory
wallet_password="$8"

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

    # echo >&2 "STORE_OUTPUT = $STORE_OUTPUT"

    # Extract txhash from STORE_OUTPUT
    TXHASH=$(echo "$STORE_OUTPUT" | jq -r '.txhash')
    echo >&2 "Transaction Hash: $TXHASH"

    sleep 15

    # Query transaction by txhash to get detailed info, including possibly the code ID
    QUERY_OUTPUT=$(archwayd query tx --type=hash "$TXHASH" --node="$node" --chain-id="$chain_id" --output json)
    # echo >&2 "QUERY_OUTPUT = $QUERY_OUTPUT"

    # Attempt to extract code ID from QUERY_OUTPUT
    CODE_ID=$(echo "$QUERY_OUTPUT" | jq -r '.logs[0].events[] | select(.type == "store_code").attributes[] | select(.key == "code_id").value')

    echo >&2 "CODE_ID = $CODE_ID"

    echo >&2 "Stored $contract_name with code ID: $CODE_ID"
    echo $CODE_ID
}

# Function to instantiate a contract
instantiate_contract() {
    local contract_name=$1
    local code_id=$2
    local instantiate_msg=$3
    echo >&2 "Instantiating $contract_name with code ID: $code_id..."
    INSTANTIATE_OUTPUT=$(yes "$wallet_password" | archwayd tx wasm instantiate $code_id "$instantiate_msg" \
        --admin="$signer_key_acc_address" \
        --from="$signer_key_acc_address" \
        --label="$contract_name" \
        --chain-id="$chain_id" \
        --node="$node" \
        --gas-prices="$gas_price" --gas=10000000 \
        --output json --yes)

    # echo >&2 "INSTANTIATE_OUTPUT = $INSTANTIATE_OUTPUT"

    # Extract txhash from INSTANTIATE_OUTPUT
    TXHASH=$(echo "$INSTANTIATE_OUTPUT" | jq -r '.txhash')
    echo >&2 "Instantiate Transaction Hash: $TXHASH"

    sleep 15 # Wait for the transaction to be processed

    # Query transaction by txhash to get detailed info, including the contract address
    QUERY_OUTPUT=$(archwayd query tx --type=hash "$TXHASH" --node="$node" --chain-id="$chain_id" --output json)
    # echo >&2 "QUERY_OUTPUT = $QUERY_OUTPUT"

    # Extract contract address from QUERY_OUTPUT
    CONTRACT_ADDRESS=$(echo "$QUERY_OUTPUT" | jq -r '.logs[0].events[] | select(.type == "instantiate").attributes[] | select(.key == "_contract_address").value')

    echo >&2 "Contract Address = $CONTRACT_ADDRESS"

    echo >&2 "Instantiated $contract_name with contract address: $CONTRACT_ADDRESS"

    echo "$CONTRACT_ADDRESS"
}

fetch_account_tracker_address() {
    local controller_contract=$1
    QUERY_OUTPUT=$(archwayd query wasm contract-state smart $controller_contract '{"query_config":{}}' --node="$node" --chain-id="$chain_id" --output json)
    ACCOUNT_TRACKER_ADDRESS=$(echo "$QUERY_OUTPUT" | jq -r '.data.config.account_tracker_address')

    echo >&2 "Account Tracker Address: $ACCOUNT_TRACKER_ADDRESS"

    echo "$ACCOUNT_TRACKER_ADDRESS"
}

# Prepare instantiation messages
instantiate_templates_msg=$(echo "$instantiate_templates_msg_template" | sed "s/SIGNER_KEY_ACC_ADDRESS/$signer_key_acc_address/g")
instantiate_controller_msg=$(echo "$instantiate_controller_msg_template" | sed "s/SIGNER_KEY_ACC_ADDRESS/$signer_key_acc_address/g")

# Store contract codes
account_contract_id=$(store_contract "warp_account")
resolver_code_id=$(store_contract "warp_resolver")
templates_code_id=$(store_contract "warp_templates")
controller_code_id=$(store_contract "warp_controller")
account_tracker_id=$(store_contract "warp_account_tracker")

# Instantiate contracts with parameters
templates_address=$(instantiate_contract "warp_templates" "$templates_code_id" "$instantiate_templates_msg")
resolver_address=$(instantiate_contract "warp_resolver" $resolver_code_id '{}')

# Update controller message with dynamic data
updated_instantiate_controller_msg=$(echo $instantiate_controller_msg | sed "s/RESOLVER_ADDRESS/$resolver_address/g" | sed "s/ACCOUNT_CONTRACT_ID/$account_contract_id/g" | sed "s/ACCOUNT_TRACKER_ID/$account_tracker_id/g")
controller_address=$(instantiate_contract "warp_controller" $controller_code_id "$updated_instantiate_controller_msg")

account_tracker_address=$(fetch_account_tracker_address $controller_address)

echo "warp-account:$account_contract_id"
echo "warp-account-tracker:$account_tracker_address:$account_tracker_id"
echo "warp-templates:$templates_address:$templates_code_id"
echo "warp-resolver:$resolver_address:$resolver_code_id"
echo "warp-controller:$controller_address:$controller_code_id"
