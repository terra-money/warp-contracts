#!/bin/bash

signer_key_acc_address="<SIGNER-ADDRESS-HERE>"
contracts_dir="$(pwd)/artifacts"
wallet_password="<WALLET-PASSWORD-HERE>"
chain_id="injective-888"
node="https://testnet.sentry.tm.injective.network:443"
gas_price="1500000000inj"
contract_name="warp_resolver" 
resolver_contract_address="inj1pku5zgartl3xdlr509u3wjfs48k2lqylpscydf"
migrate_msg='{}'

# Call the migration script with parameters
bash ./tasks/injective/migrate_warp.sh "$chain_id" "$node" "$gas_price" "$signer_key_acc_address" "$migrate_msg" "$resolver_contract_address" "$contract_name" "$contracts_dir" "$wallet_password"
