#!/bin/bash

signer_key_acc_address="<SIGNER-ADDRESS-HERE>"
contracts_dir="$(pwd)/artifacts"
wallet_password="<WALLET-PASSWORD-HERE>"
chain_id="archway-1"
node="https://rpc.mainnet.archway.io:443"
gas_price="1500000000000aarch"
contract_name="warp_resolver" 
contract_address=""
migrate_msg='{}'

# Call the migration script with parameters
bash ./tasks/archway/migrate_warp.sh "$chain_id" "$node" "$gas_price" "$signer_key_acc_address" "$migrate_msg" "$contract_address" "$contract_name" "$contracts_dir" "$wallet_password"
