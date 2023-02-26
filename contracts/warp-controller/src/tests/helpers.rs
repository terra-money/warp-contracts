use crate::contract::{instantiate, reply};
use crate::execute::account::create_account;
use crate::ContractError;
use cosmwasm_std::testing::{MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{
    Attribute, DepsMut, Env, Event, MessageInfo, OwnedDeps, Reply, Response, SubMsgResponse,
    SubMsgResult, Uint128, Uint64,
};

use warp_protocol::controller::InstantiateMsg;

pub fn instantiate_warp(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Option<String>,
    fee_collector: Option<String>,
    warp_account_code_id: Uint64,
    minimum_reward: Uint128,
    creation_fee: Uint64,
    cancellation_fee: Uint64,
    template_fee: Uint128,
    t_max: Uint64,
    t_min: Uint64,
    a_max: Uint128,
    a_min: Uint128,
    q_max: Uint64,
) -> Result<Response, ContractError> {
    let instantiate_msg = InstantiateMsg {
        owner,
        fee_collector,
        warp_account_code_id,
        minimum_reward,
        creation_fee,
        cancellation_fee,
        template_fee,
        t_max,
        t_min,
        a_max,
        a_min,
        q_max,
    };

    return instantiate(deps, env.clone(), info.clone(), instantiate_msg.clone());
}

pub fn create_warp_account(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    env: Env,
    info: MessageInfo,
    account_id: Uint64,
) -> (
    Result<Response, ContractError>,
    Result<Response, ContractError>,
) {
    let create_account_res = create_account(deps.as_mut(), env.clone(), info.clone());

    let reply_msg = Reply {
        id: 0,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![Event::new("wasm").add_attributes(vec![
                Attribute::new("action", "instantiate"),
                Attribute::new(
                    "owner",
                    format!(
                        "terra1vladvladvladvladvladvladvladvladvl{}",
                        account_id + Uint64::new(1000)
                    ),
                ),
                Attribute::new(
                    "contract_addr",
                    format!(
                        "terra1vladvladvladvladvladvladvladvladvl{}",
                        account_id + Uint64::new(2000)
                    ),
                ),
            ])],
            data: None,
        }),
    };

    let reply_res = reply(deps.as_mut(), env, reply_msg);

    return (create_account_res, reply_res);
}
