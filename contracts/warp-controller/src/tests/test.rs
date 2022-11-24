use cosmwasm_std::{to_vec, Binary, QueryRequest, StdError, WasmQuery};

#[test]
fn lol() {
    let req = QueryRequest::<String>::Wasm(WasmQuery::Smart {
        contract_addr: "terra1vlad123".to_string(),
        msg: Binary::from("{\"config\":{}}".to_string().as_bytes()),
    });

    let s = serde_json_wasm::to_string(&req).unwrap();
    println!("{}", s);

    let q = serde_json_wasm::from_str::<QueryRequest<String>>("{\"wasm\":{\"smart\":{\"contract_addr\":\"terra1vlad123\",\"msg\":\"eyJjb25maWciOnt9fQ==\"}}}").unwrap();

    let s = serde_json_wasm::to_string(&q).unwrap();
    println!("{}", s);

    let _raw = to_vec(&req)
        .map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
        })
        .unwrap();
}
