use std::env::current_dir;
use std::fs::create_dir_all;

use controller::{
    account::{AccountResponse, AccountsResponse},
    job::{JobResponse, JobsResponse},
    template::{Template, TemplateResponse, TemplatesResponse},
    QueryMsg, {Config, ConfigResponse, ExecuteMsg, InstantiateMsg, SimulateResponse},
};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cosmwasm_std::{CosmosMsg, QueryRequest};

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
    export_schema(&schema_for!(JobResponse), &out_dir);
    export_schema(&schema_for!(JobsResponse), &out_dir);
    export_schema(&schema_for!(AccountResponse), &out_dir);
    export_schema(&schema_for!(AccountsResponse), &out_dir);
    export_schema(&schema_for!(SimulateResponse), &out_dir);
    export_schema(&schema_for!(TemplateResponse), &out_dir);
    export_schema(&schema_for!(TemplatesResponse), &out_dir);
    export_schema(&schema_for!(Template), &out_dir);
    export_schema(&schema_for!(CosmosMsg), &out_dir);
    export_schema(&schema_for!(QueryRequest<String>), &out_dir);
}
