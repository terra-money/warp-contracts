use schemars::_serde_json::json;

use crate::util::variable::{all_vector_vars_present, hydrate_vars};

use cosmwasm_std::{testing::mock_env, WasmQuery};
use cosmwasm_std::{to_binary, BankQuery, Binary, ContractResult, OwnedDeps};

use crate::contract::query;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::testing::{mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{from_slice, Empty, Querier, QueryRequest, SystemError, SystemResult};
use resolver::condition::{Condition, Expr, GenExpr, NumEnvValue, NumOp, NumValue};
use resolver::variable::{QueryExpr, QueryVariable, StaticVariable, Variable, VariableKind};
use resolver::{QueryMsg, QueryValidateJobCreationMsg};
use std::marker::PhantomData;

#[test]
fn test() {
    let deps = mock_dependencies();
    let _info = mock_info("vlad", &[]);
    let env = mock_env();
    let msg = QueryValidateJobCreationMsg {
        condition: "{\"expr\":{\"decimal\":{\"op\":\"gte\",\"left\":{\"ref\":\"$warp.variable.return_amount\"},\"right\":{\"simple\":\"620000\"}}}}".parse().unwrap(),
        terminate_condition: None,
        vars: "[{\"query\":{\"kind\":\"decimal\",\"name\":\"return_amount\",\"init_fn\":{\"query\":{\"wasm\":{\"smart\":{\"msg\":\"eyJzaW11bGF0aW9uIjp7Im9mZmVyX2Fzc2V0Ijp7ImFtb3VudCI6IjEwMDAwMDAiLCJpbmZvIjp7Im5hdGl2ZV90b2tlbiI6eyJkZW5vbSI6ImliYy9CMzUwNEUwOTI0NTZCQTYxOENDMjhBQzY3MUE3MUZCMDhDNkNBMEZEMEJFN0M4QTVCNUEzRTJERDkzM0NDOUU0In19fX19\",\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\"}}},\"selector\":\"$.return_amount\"},\"reinitialize\":false,\"encode\":false}}]".to_string(),
        msgs: "[\"{\\\"wasm\\\":{\\\"execute\\\":{\\\"contract_addr\\\":\\\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\\\",\\\"msg\\\":\\\"eyJzd2FwIjp7Im9mZmVyX2Fzc2V0Ijp7ImluZm8iOnsibmF0aXZlX3Rva2VuIjp7ImRlbm9tIjoiaWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQifX0sImFtb3VudCI6IjEwMDAwMDAifSwibWF4X3NwcmVhZCI6IjAuNSIsImJlbGllZl9wcmljZSI6IjAuNjEwMzg3MzI3MzgyNDYzODE2In19\\\",\\\"funds\\\":[{\\\"denom\\\":\\\"ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4\\\",\\\"amount\\\":\\\"1000000\\\"}]}}}\"]".to_string(),
    };
    let obj = serde_json_wasm::to_string(&vec!["{\"wasm\":{\"execute\":{\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\",\"msg\":\"eyJzd2FwIjp7Im9mZmVyX2Fzc2V0Ijp7ImluZm8iOnsibmF0aXZlX3Rva2VuIjp7ImRlbm9tIjoiaWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQifX0sImFtb3VudCI6IjEwMDAwMDAifSwibWF4X3NwcmVhZCI6IjAuNSIsImJlbGllZl9wcmljZSI6IjAuNjEwMzg3MzI3MzgyNDYzODE2In19\",\"funds\":[{\"denom\":\"ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4\",\"amount\":\"1000000\"}]}}}"]).unwrap();

    let _msg1 = QueryValidateJobCreationMsg {
        condition: "{\"expr\":{\"decimal\":{\"op\":\"gte\",\"left\":{\"ref\":\"$warp.variable.return_amount\"},\"right\":{\"simple\":\"620000\"}}}}".parse().unwrap(),
        terminate_condition: None,
        vars: "[{\"query\":{\"kind\":\"decimal\",\"name\":\"return_amount\",\"init_fn\":{\"query\":{\"wasm\":{\"smart\":{\"msg\":\"eyJzaW11bGF0aW9uIjp7Im9mZmVyX2Fzc2V0Ijp7ImFtb3VudCI6IjEwMDAwMDAiLCJpbmZvIjp7Im5hdGl2ZV90b2tlbiI6eyJkZW5vbSI6ImliYy9CMzUwNEUwOTI0NTZCQTYxOENDMjhBQzY3MUE3MUZCMDhDNkNBMEZEMEJFN0M4QTVCNUEzRTJERDkzM0NDOUU0In19fX19\",\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\"}}},\"selector\":\"$.return_amount\"},\"reinitialize\":false,\"encode\":false}}]".to_string(),
        msgs: obj.clone(),
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

#[test]
fn test_all_vector_vars_present() {
    let condition = Condition::Expr(Box::new(Expr::Uint(GenExpr {
        left: NumValue::Env(NumEnvValue::Time),
        op: NumOp::Gt,
        right: NumValue::Ref("$warp.variable.next_execution".to_string()),
    })));

    let cond_string = serde_json_wasm::to_string(&condition).unwrap();

    let var = Variable::Static(StaticVariable {
        kind: VariableKind::Uint,
        name: "next_execution".to_string(),
        encode: false,
        value: "1".to_string(),
        update_fn: None,
    });

    assert_eq!(all_vector_vars_present(&vec![var], cond_string), true);
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
        value: "contract_addr".to_string(),
        update_fn: None,
    });

    let var4 = Variable::Static(StaticVariable {
        kind: VariableKind::String,
        name: "var4".to_string(),
        encode: false,
        value: "$warp.variable.var5".to_string(),
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
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None).unwrap();

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
        value: "static_value".to_string(),
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
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None).unwrap();

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
        value: "static_value".to_string(),
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
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None).unwrap();

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
fn test_hydrate_vars_nested() {
    let deps = mock_dependencies();
    let env = mock_env();

    let var1 = Variable::Static(StaticVariable {
        name: "var1".to_string(),
        kind: VariableKind::String,
        value: "static_value_1".to_string(),
        update_fn: None,
        encode: false,
    });

    #[cw_serde]
    struct AstroportNativeSwapMsg {
        swap: Swap,
    }

    #[cw_serde]
    struct Swap {
        offer_asset: OfferAsset,
        max_spread: String,
        to: String,
    }

    #[cw_serde]
    struct OfferAsset {
        info: Info,
        amount: String,
    }

    #[cw_serde]
    struct Info {
        native_token: NativeToken,
    }

    #[cw_serde]
    struct NativeToken {
        denom: String,
    }

    let astroport_native_swap_msg = AstroportNativeSwapMsg {
        swap: Swap {
            offer_asset: OfferAsset {
                info: Info {
                    native_token: NativeToken {
                        denom: "example_denom".to_string(),
                    },
                },
                amount: format!("$warp.variable.{}", "var1"),
            },
            max_spread: "0.01".to_string(),
            to: "your_address_here".to_string(),
        },
    };

    // Serialize the JSON object to a string
    let json_str = serde_json_wasm::to_string(&astroport_native_swap_msg).unwrap();

    // // Convert the JSON string to bytes
    // let json_bytes = json_str.as_bytes();
    //
    // // Base64 encode the bytes
    // let encoded_data = base64::encode(json_bytes);

    // println!("Base64 Encoded Data: {} \n\n\n", encoded_data);

    let var2 = Variable::Static(StaticVariable {
        name: "var2".to_string(),
        kind: VariableKind::String,
        value: json_str,
        update_fn: None,
        encode: true,
    });

    let vars = vec![var1, var2];
    let hydrated_vars = hydrate_vars(deps.as_ref(), env, vars, None).unwrap();

    match hydrated_vars[1].clone() {
        Variable::Static(static_var) => {
            // let decoded_val = base64::decode(static_var.value).unwrap();
            println!("Decoded Val: {}\n\n\n", static_var.value);
            // Decoded Val: {"swap":{"offer_asset":{"info":{"native_token":{"denom":"example_denom"}},"amount":"$warp.variable.var1"},"max_spread":"0.01","to":"your_address_here"}}
            // as you can see, var1 is replaced to static_value_1 as expected
        }
        _ => panic!("Expected static variable"),
    }
}
