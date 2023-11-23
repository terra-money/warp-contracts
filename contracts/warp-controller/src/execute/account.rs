use controller::CreateFundingAccountMsg;
use cosmwasm_std::{DepsMut, Env, MessageInfo, ReplyOn, Response, SubMsg, Uint64};

use crate::{state::CONFIG, util::msg::build_instantiate_warp_account_msg, ContractError};

pub fn create_funding_account(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _data: CreateFundingAccountMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let submsgs = vec![SubMsg {
        id: 2,
        msg: build_instantiate_warp_account_msg(
            Uint64::from(0u64), // placeholder
            env.contract.address.to_string(),
            config.warp_account_code_id.u64(),
            info.sender.to_string(),
            info.funds,
            None,
            None,
        ),
        gas_limit: None,
        reply_on: ReplyOn::Always,
    }];

    Ok(Response::new()
        .add_attribute("action", "create_funding_account")
        .add_submessages(submsgs))
}
