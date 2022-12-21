use cosmwasm_std::CosmosMsg;
use warp_protocol::controller::job::JobMsg;
use crate::ContractError;

pub fn jobmsg_to_cosmosmsg(msgs: Vec<JobMsg>) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut parsed_msgs = vec![];

    for msg in msgs {
        let mut vars = vec![];
        for var in msg.vars {

        }
    }

    Ok(parsed_msgs)
}