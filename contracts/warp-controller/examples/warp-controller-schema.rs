use std::env::current_dir;
use std::fs::create_dir_all;

use controller::{
    account::{LegacyAccountResponse, LegacyAccountsResponse},
    job::{JobResponse, JobsResponse},
    QueryMsg, State, StateResponse, {Config, ConfigResponse, ExecuteMsg, InstantiateMsg},
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
    export_schema(&schema_for!(ConfigResponse), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(StateResponse), &out_dir);
    export_schema(&schema_for!(JobResponse), &out_dir);
    export_schema(&schema_for!(JobsResponse), &out_dir);
    export_schema(&schema_for!(LegacyAccountResponse), &out_dir);
    export_schema(&schema_for!(LegacyAccountsResponse), &out_dir);
}
