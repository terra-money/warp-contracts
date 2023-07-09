use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
    Addr, Attribute, DepsMut, Empty, Env, Event, MessageInfo, OwnedDeps, Reply, Response,
    SubMsgResponse, SubMsgResult, Uint128, Uint64,
};

use controller::{account::CreateAccountMsg, InstantiateMsg};

use crate::{
    contract::{instantiate, reply},
    execute::account::create_account,
    ContractError,
};

pub const MIN_REWARD: u128 = 10_000;
pub const ACCOUNT_BALANCE: u128 = 20_000;

#[allow(clippy::too_many_arguments)]
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
        t_max,
        t_min,
        a_max,
        a_min,
        q_max,
    };

    instantiate(deps, env, info, instantiate_msg)
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
    let create_account_res = create_account(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        CreateAccountMsg { funds: None },
    );

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
                Attribute::new("funds", "[]"),
                Attribute::new("cw_funds", "[]"),
            ])],
            data: None,
        }),
    };

    let reply_res = reply(deps.as_mut(), env, reply_msg);

    (create_account_res, reply_res)
}

pub fn init_warp_account() -> (
    OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    Env,
    MessageInfo,
    Addr,
) {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("vlad", &[]);

    let _instantiate_res = instantiate_warp(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        Some(info.sender.to_string()),
        Some(info.sender.to_string()),
        Uint64::new(0),
        Uint128::new(MIN_REWARD),
        Uint64::new(0),
        Uint64::new(0),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .unwrap();

    let (_create_account_res, reply_res) =
        create_warp_account(&mut deps, env.clone(), info.clone(), Uint64::new(0));

    let account_addr = Addr::unchecked(get_attr(&reply_res.unwrap(), "account_address"));

    // mock account balance
    deps.querier = MockQuerier::new(&[(account_addr.as_str(), &[coin(ACCOUNT_BALANCE, "uluna")])]);

    (deps, env, info, account_addr)
}

fn get_attr(res: &Response, attr: &str) -> String {
    res.attributes
        .iter()
        .find(|attribute| attribute.key == attr)
        .unwrap()
        .value
        .to_owned()
}
