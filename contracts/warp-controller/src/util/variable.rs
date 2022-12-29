use cosmwasm_std::{CosmosMsg, Deps};
use warp_protocol::controller::variable::Variable;
use crate::ContractError;

pub fn hydrate_msgs(msgs: Vec<String>, vars: Vec<Variable>) -> Result<Vec<CosmosMsg>, ContractError> { //todo:
    let mut parsed_msgs: Vec<CosmosMsg> = vec![];
    for mut msg in msgs {
        for var in &vars {
            let (name, replacement) = match var {
                Variable::Static(v) => {
                    match v.value.clone() {
                        None => {
                            match v.default_value.clone() {
                                None => return Err(ContractError::Unauthorized {}), //todo: err
                                Some(val) => (v.name.clone(), val)
                            }
                        }
                        Some(val) => (v.name.clone(), val)
                    }
                },
                Variable::External(v) => {
                    match v.value.clone() {
                        None => {
                            match v.default_value.clone() {
                                None => return Err(ContractError::Unauthorized {}), //todo: err
                                Some(val) => (v.name.clone(), val)
                            }
                        }
                        Some(val) => (v.name.clone(), val)
                    }
                },
                Variable::Query(v) => {
                    match v.value.clone() {
                        None => {
                            match v.default_value.clone() {
                                None => return Err(ContractError::Unauthorized {}), //todo: err
                                Some(val) => (v.name.clone(), val)
                            }
                        }
                        Some(val) => (v.name.clone(), val)
                    }
                }
            };
            msg = msg.replace(&format!("\"$WARPVAR.{}\"", name), &replacement);
        }
        parsed_msgs.push(serde_json_wasm::<CosmosMsg>::from_str(&msg)?)
    }

    Ok(parsed_msgs)
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