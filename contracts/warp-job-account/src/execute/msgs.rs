use crate::ContractError;
use cosmwasm_std::{Env, Response};
use job_account::{Config, WarpMsg, WarpMsgs};

use cosmwasm_std::{CosmosMsg, DepsMut};

use super::ibc::ibc_transfer;
use super::withdraw::withdraw_assets;

pub fn execute_warp_msgs(
    deps: DepsMut,
    env: Env,
    data: WarpMsgs,
    config: Config,
) -> Result<Response, ContractError> {
    let msgs = data
        .msgs
        .into_iter()
        .flat_map(|msg| -> Vec<CosmosMsg> {
            match msg {
                WarpMsg::Generic(msg) => vec![msg],
                WarpMsg::IbcTransfer(msg) => ibc_transfer(env.clone(), msg)
                    .map(extract_messages)
                    .unwrap(),
                WarpMsg::WithdrawAssets(msg) => {
                    withdraw_assets(deps.as_ref(), env.clone(), msg, config.clone())
                        .map(extract_messages)
                        .unwrap()
                }
            }
        })
        .collect::<Vec<CosmosMsg>>();

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "warp_msgs"))
}

fn extract_messages(resp: Response) -> Vec<CosmosMsg> {
    resp.messages
        .into_iter()
        .map(|cosmos_msg| cosmos_msg.msg)
        .collect()
}
