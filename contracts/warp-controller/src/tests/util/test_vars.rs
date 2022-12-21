use crate::util::condition::resolve_cond;
use crate::util::path::resolve_path;
use json_codec_wasm::ast::Ref;
use json_codec_wasm::{Decoder, Encoder, Json};
use std::io::Cursor;

#[test]
fn test_vars() {
    let mut test_msg = "{\"execute\":{\"test\":}}".to_string();

    let query_response = "{\"response\":{\"test\": \"1\"}}";

    let j = Decoder::default(query_response.chars()).decode().unwrap();

    let r = Ref::new(&j);

    let mut c = Cursor::new(Vec::new());
    let mut e = Encoder::new(&mut c);

    e.encode(r.get("response").get("test").value().unwrap());

    let injected_json = String::from_utf8(c.into_inner()).unwrap();
    let injected_idx = 19 as usize;

    String::insert_str(&mut test_msg, injected_idx, injected_json.as_str());

    println!("{}", test_msg);
}
