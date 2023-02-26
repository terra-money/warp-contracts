use warp_protocol::controller::{
    condition::{Condition, Expr, GenExpr, NumOp, NumValue},
    variable::{StaticVariable, Variable, VariableKind},
};

use crate::util::variable::all_vector_vars_present;

#[test]
fn test_vars() {
    let test_msg = "{\"execute\":{\"test\":\"$WARPVAR.test\"}}".to_string();

    let _idx = test_msg.find("\"$WARPVAR\"");

    let _new_str = test_msg.replace("\"$WARPVAR.test\"", "\"input\"");

    // test_msg.repl

    // let query_response = "{\"response\":{\"test\": \"1\"}}";
    //
    // let j = Decoder::default(query_response.chars()).decode().unwrap();
    //
    // let r = Ref::new(&j);
    //
    // let mut c = Cursor::new(Vec::new());
    // let mut e = Encoder::new(&mut c);
    //
    // e.encode(r.get("response").get("test").value().unwrap());
    //
    // let injected_json = String::from_utf8(c.into_inner()).unwrap();
    // let injected_idx = 19 as usize;
    //
    // String::insert_str(&mut test_msg, injected_idx, injected_json.as_str());

    println!("{}", test_msg);
}

#[test]
fn test_all_vector_vars_present() {
    let condition = Condition::Expr(Box::new(Expr::Uint(GenExpr {
        left: NumValue::Env(warp_protocol::controller::condition::NumEnvValue::Time),
        op: NumOp::Gt,
        right: NumValue::Ref("$warp.variable.next_execution".to_string()),
    })));

    let cond_string = serde_json_wasm::to_string(&condition).unwrap();

    let var = Variable::Static(StaticVariable {
        kind: VariableKind::Uint,
        name: "next_execution".to_string(),
        value: "1".to_string(),
        update_fn: None,
    });

    assert_eq!(all_vector_vars_present(&vec![var], cond_string), true);
}
