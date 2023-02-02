use cosmwasm_std::{to_vec, Binary, QueryRequest, StdError, WasmQuery};

#[test]
fn lol() {
    let v = "\"32000\"";
    let v_split = &v[1..v.len()-1];
    println!("{}", v_split)
}
