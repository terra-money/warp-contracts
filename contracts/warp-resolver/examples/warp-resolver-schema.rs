use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cosmwasm_std::{CosmosMsg, QueryRequest};
use resolver::{ExecuteMsg, InstantiateMsg, QueryMsg, variable::Variable, condition::Condition, SimulateResponse, ResolveResponse};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(Variable), &out_dir);
    export_schema(&schema_for!(Condition), &out_dir);
    export_schema(&schema_for!(SimulateResponse), &out_dir);
    export_schema(&schema_for!(ResolveResponse), &out_dir);
    export_schema(&schema_for!(CosmosMsg), &out_dir);
    export_schema(&schema_for!(QueryRequest<String>), &out_dir);
}
