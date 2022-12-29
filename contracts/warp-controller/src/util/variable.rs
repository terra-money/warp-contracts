use std::ops::Deref;
use cosmwasm_std::CosmosMsg;
use warp_protocol::controller::variable::Variable;
use crate::ContractError;

pub fn hydrate_msgs(msgs: Vec<String>) -> Result<Vec<CosmosMsg>, ContractError> { //todo:
    // let mut parsed_msgs = vec![];

    // for msg in msgs {
    //     let mut vars = vec![];
    //     for var in msg.vars {
    //
    //     }
    // }

    Ok(vec![])
}

pub fn get_var(name: String, vars: &Vec<Variable>) -> Result<&Variable, ContractError> {
    for var in vars {
        let n = match var {
            Variable::Static(v) => v.name.clone(),
            Variable::External(v) => v.name.clone(),
            Variable::Query(v) => v.name.clone(),
        };
        if n == name {
            return Ok(var)
        }
    }
    Err(ContractError::Unauthorized {})//todo: err
}