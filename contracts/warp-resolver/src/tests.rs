use crate::contract::query;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use resolver::{QueryMsg, QueryValidateJobCreationMsg};

#[test]
fn test() {
    let deps = mock_dependencies();
    let _info = mock_info("vlad", &[]);
    let env = mock_env();
    let msg = QueryValidateJobCreationMsg {
        condition: "{\"expr\":{\"decimal\":{\"op\":\"gte\",\"left\":{\"ref\":\"$warp.variable.return_amount\"},\"right\":{\"simple\":\"620000\"}}}}".parse().unwrap(),
        vars: "[{\"query\":{\"kind\":\"decimal\",\"name\":\"return_amount\",\"init_fn\":{\"query\":{\"wasm\":{\"smart\":{\"msg\":\"eyJzaW11bGF0aW9uIjp7Im9mZmVyX2Fzc2V0Ijp7ImFtb3VudCI6IjEwMDAwMDAiLCJpbmZvIjp7Im5hdGl2ZV90b2tlbiI6eyJkZW5vbSI6ImliYy9CMzUwNEUwOTI0NTZCQTYxOENDMjhBQzY3MUE3MUZCMDhDNkNBMEZEMEJFN0M4QTVCNUEzRTJERDkzM0NDOUU0In19fX19\",\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\"}}},\"selector\":\"$.return_amount\"},\"reinitialize\":false}}]".to_string(),
        msgs: "[\"{\\\"wasm\\\":{\\\"execute\\\":{\\\"contract_addr\\\":\\\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\\\",\\\"msg\\\":\\\"eyJzd2FwIjp7Im9mZmVyX2Fzc2V0Ijp7ImluZm8iOnsibmF0aXZlX3Rva2VuIjp7ImRlbm9tIjoiaWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQifX0sImFtb3VudCI6IjEwMDAwMDAifSwibWF4X3NwcmVhZCI6IjAuNSIsImJlbGllZl9wcmljZSI6IjAuNjEwMzg3MzI3MzgyNDYzODE2In19\\\",\\\"funds\\\":[{\\\"denom\\\":\\\"ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4\\\",\\\"amount\\\":\\\"1000000\\\"}]}}}\"]".to_string(),
    };
    let obj = serde_json_wasm::to_string(&vec!["{\"wasm\":{\"execute\":{\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\",\"msg\":\"eyJzd2FwIjp7Im9mZmVyX2Fzc2V0Ijp7ImluZm8iOnsibmF0aXZlX3Rva2VuIjp7ImRlbm9tIjoiaWJjL0IzNTA0RTA5MjQ1NkJBNjE4Q0MyOEFDNjcxQTcxRkIwOEM2Q0EwRkQwQkU3QzhBNUI1QTNFMkREOTMzQ0M5RTQifX0sImFtb3VudCI6IjEwMDAwMDAifSwibWF4X3NwcmVhZCI6IjAuNSIsImJlbGllZl9wcmljZSI6IjAuNjEwMzg3MzI3MzgyNDYzODE2In19\",\"funds\":[{\"denom\":\"ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4\",\"amount\":\"1000000\"}]}}}"]).unwrap();

    let _msg1 = QueryValidateJobCreationMsg {
        condition: "{\"expr\":{\"decimal\":{\"op\":\"gte\",\"left\":{\"ref\":\"$warp.variable.return_amount\"},\"right\":{\"simple\":\"620000\"}}}}".parse().unwrap(),
        vars: "[{\"query\":{\"kind\":\"decimal\",\"name\":\"return_amount\",\"init_fn\":{\"query\":{\"wasm\":{\"smart\":{\"msg\":\"eyJzaW11bGF0aW9uIjp7Im9mZmVyX2Fzc2V0Ijp7ImFtb3VudCI6IjEwMDAwMDAiLCJpbmZvIjp7Im5hdGl2ZV90b2tlbiI6eyJkZW5vbSI6ImliYy9CMzUwNEUwOTI0NTZCQTYxOENDMjhBQzY3MUE3MUZCMDhDNkNBMEZEMEJFN0M4QTVCNUEzRTJERDkzM0NDOUU0In19fX19\",\"contract_addr\":\"terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr\"}}},\"selector\":\"$.return_amount\"},\"reinitialize\":false}}]".to_string(),
        msgs: obj.clone(),
    };

    println!("{}", serde_json_wasm::to_string(&obj).unwrap());

    let test = query(deps.as_ref(), env, QueryMsg::QueryValidateJobCreation(msg)).unwrap();
    println!("{}", test)
}
