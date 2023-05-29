use std::env::current_dir;
use std::fs::create_dir_all;

use account::{Config, ExecuteMsg, InstantiateMsg};
use controller::{
    account::{AccountResponse, AccountsResponse},
    job::{JobResponse, JobsResponse},
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(Config), &out_dir);
    export_schema(&schema_for!(JobResponse), &out_dir);
    export_schema(&schema_for!(JobsResponse), &out_dir);
    export_schema(&schema_for!(AccountResponse), &out_dir);
    export_schema(&schema_for!(AccountsResponse), &out_dir);
}
