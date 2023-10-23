use crate::ContractError;
use cosmwasm_std::CosmosMsg::Stargate;
use cosmwasm_std::{Env, Response};
use job_account::{IbcTransferMsg, TimeoutBlock};
use prost::Message;

pub fn ibc_transfer(env: Env, data: IbcTransferMsg) -> Result<Response, ContractError> {
    let mut transfer_msg = data.transfer_msg.clone();

    if data.timeout_block_delta.is_some() && data.transfer_msg.timeout_block.is_some() {
        let block = transfer_msg.timeout_block.unwrap();
        transfer_msg.timeout_block = Some(TimeoutBlock {
            revision_number: Some(block.revision_number()),
            revision_height: Some(env.block.height + data.timeout_block_delta.unwrap()),
        })
    }

    if data.timeout_timestamp_seconds_delta.is_some() {
        transfer_msg.timeout_timestamp = Some(
            env.block
                .time
                .plus_seconds(
                    env.block.time.seconds() + data.timeout_timestamp_seconds_delta.unwrap(),
                )
                .nanos(),
        );
    }

    Ok(Response::new().add_message(Stargate {
        type_url: "/ibc.applications.transfer.v1.MsgTransfer".to_string(),
        value: transfer_msg.encode_to_vec().into(),
    }))
}
