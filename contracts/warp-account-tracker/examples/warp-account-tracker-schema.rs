use std::env::current_dir;
use std::fs::create_dir_all;

use account_tracker::{
    Account, AccountResponse, AccountsResponse, Config, ConfigResponse, ExecuteMsg,
    FundingAccountResponse, FundingAccountsResponse, InstantiateMsg, QueryMsg,
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(AccountsResponse), &out_dir);
    export_schema(&schema_for!(AccountResponse), &out_dir);
    export_schema(&schema_for!(FundingAccountResponse), &out_dir);
    export_schema(&schema_for!(FundingAccountsResponse), &out_dir);
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(Account), &out_dir);
}
