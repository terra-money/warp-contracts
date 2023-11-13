use crate::ContractError;
use controller::account::{WarpMsg, WarpMsgs};
use cosmwasm_std::{Deps, Env, Response};
use job_account::Config;

use cosmwasm_std::{CosmosMsg, DepsMut};

use super::ibc::ibc_transfer;
use super::withdraw::withdraw_assets;

pub fn execute_warp_msgs(
    deps: DepsMut,
    env: Env,
    data: WarpMsgs,
    config: Config,
) -> Result<Response, ContractError> {
    let msgs = warp_msgs_to_cosmos_msgs(deps.as_ref(), env, data.msgs, config).unwrap();

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "warp_msgs"))
}

pub fn warp_msgs_to_cosmos_msgs(
    deps: Deps,
    env: Env,
    msgs: Vec<WarpMsg>,
    config: Config,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let result = msgs
        .into_iter()
        .flat_map(|msg| -> Vec<CosmosMsg> {
            match msg {
                WarpMsg::Generic(msg) => vec![msg],
                WarpMsg::IbcTransfer(msg) => ibc_transfer(env.clone(), msg)
                    .map(extract_messages)
                    .unwrap(),
                WarpMsg::WithdrawAssets(msg) => {
                    withdraw_assets(deps, env.clone(), msg, config.clone())
                        .map(extract_messages)
                        .unwrap()
                }
            }
        })
        .collect::<Vec<CosmosMsg>>();

    Ok(result)
}

fn extract_messages(resp: Response) -> Vec<CosmosMsg> {
    resp.messages
        .into_iter()
        .map(|cosmos_msg| cosmos_msg.msg)
        .collect()
}
