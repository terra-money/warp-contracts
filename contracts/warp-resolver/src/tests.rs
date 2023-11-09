use controller::job::Execution;
use resolver::condition::{NumValue, StringEnvValue, StringValue};
use schemars::_serde_json::json;

use crate::util::variable::{hydrate_msgs, hydrate_vars};

use cosmwasm_std::{testing::mock_env, WasmQuery};
use cosmwasm_std::{
    to_binary, BankQuery, Binary, ContractResult, CosmosMsg, OwnedDeps, Uint256, WasmMsg,
};

use crate::contract::query;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{from_slice, Empty, Querier, QueryRequest, SystemError, SystemResult};

use resolver::variable::{
    FnValue, QueryExpr, QueryVariable, StaticVariable, Variable, VariableKind,
};
use resolver::{QueryMsg, QueryValidateJobCreationMsg};
use std::marker::PhantomData;

#[cw_serde]
struct TestStruct {
    test: String,
}

#[test]
fn test() {
    let deps = mock_dependencies();
    let _info = mock_info("vlad", &[]);
    let env = mock_env();
    let msg = QueryValidateJobCreationMsg {
        executions: vec![Execution {
            condition: "{\"expr\":{\"decimal\":{\"op\":\"gte\",\"left\":{\"ref\":\"$warp.variable.return_amount\"},\"right\":{\"simple\":\"620000\"}}}}".parse().unwrap(),
            msgs: "[{\"wasm\":{\"execute\":{\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\",\"msg\":\"eyJzd2FwIjp7Im9mZmVyX2Fzc2V0Ijp7ImluZm8iOnsibmF0aXZlX3Rva2VuIjp7ImRlbm9tIjoiaWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQifX0sImFtb3VudCI6IjEwMDAwMDAifSwibWF4X3NwcmVhZCI6IjAuNSIsImJlbGllZl9wcmljZSI6IjAuNjEwMzg3MzI3MzgyNDYzODE2In19\",\"funds\":[{\"denom\":\"ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4\",\"amount\":\"1000000\"}]}}}]".to_string(),    
        }],
        terminate_condition: None,
        vars: "[{\"query\":{\"kind\":\"decimal\",\"name\":\"return_amount\",\"init_fn\":{\"query\":{\"wasm\":{\"smart\":{\"msg\":\"eyJzaW11bGF0aW9uIjp7Im9mZmVyX2Fzc2V0Ijp7ImFtb3VudCI6IjEwMDAwMDAiLCJpbmZvIjp7Im5hdGl2ZV90b2tlbiI6eyJkZW5vbSI6ImliYy9CMzUwNEUwOTI0NTZCQTYxOENDMjhBQzY3MUE3MUZCMDhDNkNBMEZEMEJFN0M4QTVCNUEzRTJERDkzM0NDOUU0In19fX19\",\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\"}}},\"selector\":\"$.return_amount\"},\"reinitialize\":false,\"encode\":false}}]".to_string(),
    };
    let obj = serde_json_wasm::to_string(&vec!["{\"wasm\":{\"execute\":{\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\",\"msg\":\"eyJzd2FwIjp7Im9mZmVyX2Fzc2V0Ijp7ImluZm8iOnsibmF0aXZlX3Rva2VuIjp7ImRlbm9tIjoiaWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQifX0sImFtb3VudCI6IjEwMDAwMDAifSwibWF4X3NwcmVhZCI6IjAuNSIsImJlbGllZl9wcmljZSI6IjAuNjEwMzg3MzI3MzgyNDYzODE2In19\",\"funds\":[{\"denom\":\"ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4\",\"amount\":\"1000000\"}]}}}"]).unwrap();

    let _msg1 = QueryValidateJobCreationMsg {
        terminate_condition: None,
        vars: "[{\"query\":{\"kind\":\"decimal\",\"name\":\"return_amount\",\"init_fn\":{\"query\":{\"wasm\":{\"smart\":{\"msg\":\"eyJzaW11bGF0aW9uIjp7Im9mZmVyX2Fzc2V0Ijp7ImFtb3VudCI6IjEwMDAwMDAiLCJpbmZvIjp7Im5hdGl2ZV90b2tlbiI6eyJkZW5vbSI6ImliYy9CMzUwNEUwOTI0NTZCQTYxOENDMjhBQzY3MUE3MUZCMDhDNkNBMEZEMEJFN0M4QTVCNUEzRTJERDkzM0NDOUU0In19fX19\",\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\"}}},\"selector\":\"$.return_amount\"},\"reinitialize\":false,\"encode\":false}}]".to_string(),
        executions: vec![Execution {
            condition: "{\"expr\":{\"decimal\":{\"op\":\"gte\",\"left\":{\"ref\":\"$warp.variable.return_amount\"},\"right\":{\"simple\":\"620000\"}}}}".parse().unwrap(),
            msgs: obj.clone(),
        }],
    };

    println!("{}", serde_json_wasm::to_string(&obj).unwrap());

    let test = query(deps.as_ref(), env, QueryMsg::QueryValidateJobCreation(msg)).unwrap();
    println!("{}", test)
}

#[test]
fn test_vars() {
    let test_msg = "{\"execute\":{\"test\":\"$WARPVAR.test\"}}".to_string();

    let _idx = test_msg.find("\"$WARPVAR\"");

    let _new_str = test_msg.replace("\"$WARPVAR.test\"", "\"input\"");
}

pub fn mock_dependencies() -> OwnedDeps<MockStorage, MockApi, WasmMockQuerier> {
    let custom_querier: WasmMockQuerier = WasmMockQuerier::new(MockQuerier::new(&[]));

    OwnedDeps {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: custom_querier,
        custom_query_type: PhantomData,
    }
}

pub struct WasmMockQuerier {
    base: MockQuerier<Empty>,
}

impl Querier for WasmMockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> SystemResult<ContractResult<Binary>> {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };
        self.handle_query(&request)
    }
}

impl WasmMockQuerier {
    pub fn handle_query(
        &self,
        request: &QueryRequest<Empty>,
    ) -> SystemResult<ContractResult<Binary>> {
        match &request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg: _,
            }) => {
                // Mock logic for the Wasm::Smart case
                // Here for simplicity, we return the contract_addr and msg as is.

                // Mock logic for the Wasm::Smart case
                // Here we return a JSON object with "address" and "msg" fields.
                let response: String = json!({
                    "address": contract_addr,
                    "msg": "Mock message"
                })
                .to_string();

                SystemResult::Ok(ContractResult::Ok(to_binary(&response).unwrap()))
            }
            QueryRequest::Bank(BankQuery::Balance {
                address: contract_addr,
                denom: _,
            }) => SystemResult::Ok(ContractResult::Ok(
                to_binary(&contract_addr.to_string()).unwrap(),
            )),
            _ => self.base.handle_query(request),
        }
    }
}

impl WasmMockQuerier {
    pub fn new(base: MockQuerier<Empty>) -> Self {
        WasmMockQuerier { base }
    }
}

#[test]
fn test_hydrate_vars_nested_variables_binary_json() {
    let deps = mock_dependencies();
    let env = mock_env();

    let var5 = Variable::Static(StaticVariable {
        kind: VariableKind::String,
        name: "var5".to_string(),
        encode: false,
        value: None,
        init_fn: FnValue::String(StringValue::Simple("contract_addr".to_string())),
        reinitialize: false,
        update_fn: None,
    });

    let var4 = Variable::Static(StaticVariable {
        kind: VariableKind::String,
        name: "var4".to_string(),
        encode: false,
        value: None,
        init_fn: FnValue::String(StringValue::Ref("$warp.variable.var5".to_string())),
        reinitialize: false,
        update_fn: None,
    });

    let var3 = Variable::Query(QueryVariable {
        name: "var3".to_string(),
        kind: VariableKind::Json,
        init_fn: QueryExpr {
            selector: "$".to_string(),
            query: QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: "contract_addr".to_string(),
                msg: Binary::from(r#"{"test":"test"}"#.as_bytes()),
            }),
        },
        value: None,
        reinitialize: false,
        update_fn: None,
        encode: true,
    });

    let var1 = Variable::Query(QueryVariable {
        name: "var1".to_string(),
        kind: VariableKind::Json,
        init_fn: QueryExpr {
            selector: "$".to_string(),
            query: QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: "contract_addr".to_string(),
                msg: Binary::from(r#"{"test":"$warp.variable.var3"}"#.as_bytes()),
            }),
        },
        value: None,
        reinitialize: false,
        update_fn: None,
        encode: true,
    });

    let var2 = Variable::Query(QueryVariable {
        name: "var2".to_string(),
        kind: VariableKind::Json,
        init_fn: QueryExpr {
            selector: "$".to_string(),
            query: QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: "$warp.variable.var4".to_string(),
                msg: Binary::from(r#"{"test":"$warp.variable.var1"}"#.as_bytes()),
            }),
        },
        value: None,
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let vars = vec![var5, var4, var3, var1, var2];
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None, None).unwrap();

    assert_eq!(
        hydrated_vars[4],
        Variable::Query(QueryVariable {
            name: "var2".to_string(),
            kind: VariableKind::Json,
            init_fn: QueryExpr {
                selector: "$".to_string(),
                query: QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: "contract_addr".to_string(),
                    msg: Binary::from(
                        r#"{"test":"eyJhZGRyZXNzIjoiY29udHJhY3RfYWRkciIsIm1zZyI6Ik1vY2sgbWVzc2FnZSJ9"}"#.as_bytes()
                    ),
                }),
            },
            value: Some(r#"{"address":"contract_addr","msg":"Mock message"}"#.to_string()),
            reinitialize: false,
            update_fn: None,
            encode: false,
        })
    );
}

#[test]
fn test_hydrate_vars_nested_variables_binary() {
    let deps = mock_dependencies();
    let env = mock_env();

    let var1 = Variable::Static(StaticVariable {
        name: "var1".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple("static_value".to_string())),
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let init_fn = QueryExpr {
        selector: "$".to_string(),
        query: QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: "$warp.variable.var1".to_string(),
            msg: Binary::from(r#"{"test": "$warp.variable.var1"}"#.as_bytes()),
        }),
    };

    let var2 = Variable::Query(QueryVariable {
        name: "var2".to_string(),
        kind: VariableKind::String,
        init_fn,
        value: None,
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let vars = vec![var1, var2];
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None, None).unwrap();

    assert_eq!(
        hydrated_vars[1],
        Variable::Query(QueryVariable {
            name: "var2".to_string(),
            kind: VariableKind::String,
            init_fn: QueryExpr {
                selector: "$".to_string(),
                query: QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: "static_value".to_string(),
                    msg: Binary::from(r#"{"test": "static_value"}"#.as_bytes()),
                }),
            },
            value: Some(r#"{"address":"static_value","msg":"Mock message"}"#.to_string()),
            reinitialize: false,
            update_fn: None,
            encode: false,
        })
    );
}
#[test]
fn test_hydrate_vars_nested_variables_non_binary() {
    let deps = mock_dependencies();
    let env = mock_env();

    let var1 = Variable::Static(StaticVariable {
        name: "var1".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple("static_value".to_string())),
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let init_fn = QueryExpr {
        selector: "$".to_string(),
        query: QueryRequest::Bank(BankQuery::Balance {
            address: "$warp.variable.var1".to_string(),
            denom: "denom".to_string(),
        }),
    };

    let var2 = Variable::Query(QueryVariable {
        name: "var2".to_string(),
        kind: VariableKind::String,
        init_fn,
        value: None,
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let vars = vec![var1, var2];
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None, None).unwrap();

    assert_eq!(
        hydrated_vars[1],
        Variable::Query(QueryVariable {
            name: "var2".to_string(),
            kind: VariableKind::String,
            init_fn: QueryExpr {
                selector: "$".to_string(),
                query: QueryRequest::Bank(BankQuery::Balance {
                    address: "static_value".to_string(),
                    denom: "denom".to_string(),
                }),
            },
            value: Some("static_value".to_string()),
            reinitialize: false,
            update_fn: None,
            encode: false,
        })
    );
}

#[test]
fn test_hydrate_static_nested_vars_and_hydrate_msgs() {
    let deps = mock_dependencies();
    let env = mock_env();

    let var1 = Variable::Static(StaticVariable {
        name: "var1".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple("static_value_1".to_string())),
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    // ============ TEST HYDRATED VALUE  ============

    let test_msg = TestStruct {
        test: format!("$warp.variable.{}", "var1"),
    };

    let json_str = serde_json_wasm::to_string(&test_msg).unwrap();

    let raw_str = r#"{"test":"static_value_1"}"#.to_string();

    let var2 = Variable::Static(StaticVariable {
        name: "var2".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple(json_str.clone())),
        reinitialize: false,
        update_fn: None,
        // when encode is false, value will not be base64 encoded after msgs hydration
        encode: false,
    });

    let vars = vec![var1.clone(), var2];
    let hydrated_vars = hydrate_vars(deps.as_ref(), env.clone(), vars, None, None).unwrap();
    let hydrated_var1 = hydrated_vars[0].clone();
    let hydrated_var2 = hydrated_vars[1].clone();
    match hydrated_var2.clone() {
        Variable::Static(static_var) => {
            // var3.encode = false doesn't matter here, it only matters when injecting to msgs during msg hydration
            assert_eq!(
                String::from_utf8(static_var.value.unwrap_or_default().into()).unwrap(),
                raw_str
            )
        }
        _ => panic!("Expected static variable"),
    };

    let var3 = Variable::Static(StaticVariable {
        name: "var3".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple(json_str)),
        reinitialize: false,
        update_fn: None,
        // when encode is true, value will be base64 encoded after msgs hydration
        encode: true,
    });

    let vars = vec![var1, var3];
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None, None).unwrap();
    let hydrated_var3 = hydrated_vars[1].clone();
    match hydrated_var3.clone() {
        Variable::Static(static_var) => {
            // var3.encode = true doesn't matter here, it only matters when injecting to msgs during msg hydration
            assert_eq!(
                String::from_utf8(static_var.value.unwrap_or_default().into()).unwrap(),
                raw_str
            );
        }
        _ => panic!("Expected static variable"),
    };

    // ============ TEST HYDRATED MSG AND VAR VALUE SHOULD BE ENCODED ACCORDINGLY ============

    let encoded_val = base64::encode(raw_str.clone());
    assert_eq!(encoded_val, "eyJ0ZXN0Ijoic3RhdGljX3ZhbHVlXzEifQ==");
    let msgs =
        r#"[{"wasm":{"execute":{"contract_addr":"$warp.variable.var1","msg":"eyJ0ZXN0Ijoic3RhdGljX3ZhbHVlXzEifQ==","funds":[]}}},
        {"wasm":{"execute":{"contract_addr":"$warp.variable.var3","msg":"$warp.variable.var3","funds":[]}}}]"#
            .to_string();

    let hydrated_msgs =
        hydrate_msgs(msgs, vec![hydrated_var1, hydrated_var2, hydrated_var3]).unwrap();

    assert_eq!(
        hydrated_msgs[0],
        CosmosMsg::Wasm(WasmMsg::Execute {
            // Because var1.encode = false, contract_addr should use the plain text value
            contract_addr: "static_value_1".to_string(),
            msg: Binary::from(raw_str.as_bytes()),
            funds: vec![]
        })
    );

    assert_eq!(
        hydrated_msgs[1],
        CosmosMsg::Wasm(WasmMsg::Execute {
            // Because var3.encode = true, contract_addr should use the encoded value
            contract_addr: encoded_val,
            // msg is not Binary::from(encoded_val.as_bytes()) appears to be a cosmos msg thing, not a warp thing
            msg: Binary::from(raw_str.as_bytes()),
            funds: vec![]
        })
    )
}

#[test]
fn test_hydrate_static_env_vars_and_hydrate_msgs() {
    let deps = mock_dependencies();
    let env = mock_env();

    let dummy_warp_account_addr = "terra1".to_string();

    let json_str = serde_json_wasm::to_string(&TestStruct {
        test: format!("$warp.variable.{}", "var2"),
    })
    .unwrap();

    let raw_str = r#"{"test":"100"}"#.to_string();

    let encoded_val = base64::encode(raw_str.clone());
    assert_eq!(encoded_val, "eyJ0ZXN0IjoiMTAwIn0=");

    // ============ TEST HYDRATED VALUE  ============

    let var1 = Variable::Static(StaticVariable {
        name: "var1".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple("static_value_1".to_string())),
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let var2 = Variable::Static(StaticVariable {
        name: "var2".to_string(),
        kind: VariableKind::Uint,
        value: None,
        init_fn: FnValue::Uint(NumValue::Simple(Uint256::from(100_u64))),
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let var3 = Variable::Static(StaticVariable {
        name: "var3".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Simple(json_str)),
        reinitialize: false,
        update_fn: None,
        encode: true,
    });

    let var4 = Variable::Static(StaticVariable {
        name: "var4".to_string(),
        kind: VariableKind::String,
        value: None,
        init_fn: FnValue::String(StringValue::Env(StringEnvValue::WarpAccountAddr)),
        reinitialize: false,
        update_fn: None,
        encode: false,
    });

    let vars = vec![var1, var2, var3, var4];
    let hydrated_vars = hydrate_vars(
        deps.as_ref(),
        env,
        vars,
        None,
        Some(dummy_warp_account_addr.clone()),
    )
    .unwrap();

    let hydrated_var1 = hydrated_vars[0].clone();
    let hydrated_var2 = hydrated_vars[1].clone();
    match hydrated_var2.clone() {
        Variable::Static(static_var) => {
            assert_eq!(
                String::from_utf8(static_var.value.unwrap_or_default().into()).unwrap(),
                "100".to_string()
            )
        }
        _ => panic!("Expected static variable"),
    };
    let hydrated_var3 = hydrated_vars[2].clone();
    match hydrated_var3.clone() {
        Variable::Static(static_var) => {
            assert_eq!(
                String::from_utf8(static_var.value.unwrap_or_default().into()).unwrap(),
                raw_str
            )
        }
        _ => panic!("Expected static variable"),
    };
    let hydrated_var4 = hydrated_vars[3].clone();
    match hydrated_var4.clone() {
        Variable::Static(static_var) => {
            assert_eq!(
                String::from_utf8(static_var.value.unwrap_or_default().into()).unwrap(),
                dummy_warp_account_addr
            )
        }
        _ => panic!("Expected static variable"),
    };

    // ============ TEST HYDRATED MSG AND VAR VALUE SHOULD BE ENCODED ACCORDINGLY ============

    let msgs =
        r#"[
            {"wasm":{"execute":{"contract_addr":"$warp.variable.var1","msg":"eyJ0ZXN0IjoiMTAwIn0=","funds":[]}}},
            {"wasm":{"execute":{"contract_addr":"$warp.variable.var4","msg":"$warp.variable.var3","funds":[]}}}
        ]"#
            .to_string();

    let hydrated_msgs = hydrate_msgs(
        msgs,
        vec![hydrated_var1, hydrated_var2, hydrated_var3, hydrated_var4],
    )
    .unwrap();

    assert_eq!(
        hydrated_msgs[0],
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "static_value_1".to_string(),
            msg: Binary::from(raw_str.as_bytes()),
            funds: vec![]
        })
    );

    assert_eq!(
        hydrated_msgs[1],
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: dummy_warp_account_addr,
            msg: Binary::from(raw_str.as_bytes()),
            funds: vec![]
        })
    )
}
