#!/bin/bash

signer_key_acc_address="<SIGNER-ADDRESS-HERE>"
contracts_dir="$(pwd)/artifacts"
wallet_password="<WALLET-PASSWORD-HERE>"
chain_id="Oraichain-testnet"
node="https://testnet-rpc.orai.io:443"
gas_price="0.005orai"

# Instantiate message templates with the signer key address dynamically replaced
instantiate_templates_msg=$(echo '{"owner":"SIGNER_KEY_ACC_ADDRESS","fee_collector":"SIGNER_KEY_ACC_ADDRESS","templates":[],"fee_denom":"orai"}' | sed "s/SIGNER_KEY_ACC_ADDRESS/$signer_key_acc_address/g")
instantiate_controller_msg=$(echo '{"fee_denom":"orai","fee_collector":"SIGNER_KEY_ACC_ADDRESS","warp_account_code_id":"ACCOUNT_CONTRACT_ID","account_tracker_code_id":"ACCOUNT_TRACKER_ID","minimum_reward":"10000","cancellation_fee_rate":"5","resolver_address":"RESOLVER_ADDRESS","creation_fee_min":"50000","creation_fee_max":"10000000","burn_fee_min":"25000","maintenance_fee_min":"25000","maintenance_fee_max":"1000000","duration_days_min":"7","duration_days_max":"90","duration_days_limit":"180","queue_size_left":"5000","queue_size_right":"50000","burn_fee_rate":"25"}' | sed "s/SIGNER_KEY_ACC_ADDRESS/$signer_key_acc_address/g")

# Call the deployment script with parameters
bash ./tasks/oraichain/deploy_warp.sh "$chain_id" "$node" "$gas_price" "$signer_key_acc_address" "$instantiate_templates_msg" "$instantiate_controller_msg" "$contracts_dir" "$wallet_password"
