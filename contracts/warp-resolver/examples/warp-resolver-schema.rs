use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use resolver::{
    Config, ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, Template, TemplateResponse,
    TemplatesResponse,
};

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
    export_schema(&schema_for!(TemplateResponse), &out_dir);
    export_schema(&schema_for!(TemplatesResponse), &out_dir);
    export_schema(&schema_for!(Template), &out_dir);
}
