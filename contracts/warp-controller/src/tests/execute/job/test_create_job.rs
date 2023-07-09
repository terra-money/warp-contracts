use cosmwasm_std::{
    coin,
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, BankMsg, CosmosMsg, DistributionMsg, GovMsg, IbcMsg, IbcTimeout, IbcTimeoutBlock,
    ReplyOn, Response, StakingMsg, SubMsg, Uint128, Uint64, VoteOption, WasmMsg,
};

use crate::{
    contract::execute,
    tests::helpers::{init_warp_account, instantiate_warp, ACCOUNT_BALANCE, MIN_REWARD},
    ContractError,
};

use account::GenericMsg;
use controller::{
    condition::{BlockExpr, Condition, Expr, NumOp},
    job::CreateJobMsg,
    variable::{StaticVariable, Variable, VariableKind},
    ExecuteMsg,
};

#[test]
fn test_create_job_success() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "send funds job".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap();

    let expected = Response::new()
        .add_attributes(vec![
            ("action", "create_job"),
            ("job_id", "1"),
            ("job_owner", "terra1vladvladvladvladvladvladvladvladvl1000"),
            ("job_name", "send funds job"),
            ("job_status", "\"Pending\""),
            ("job_condition", "{\"expr\":{\"block_height\":{\"comparator\":\"42\",\"op\":\"gte\"}}}"),
            ("job_msgs", "[\"{\\\"bank\\\":{\\\"send\\\":{\\\"to_address\\\":\\\"vlad\\\",\\\"amount\\\":[{\\\"denom\\\":\\\"uluna\\\",\\\"amount\\\":\\\"1000\\\"}]}}}\"]"),
            ("job_reward", "10000"),
            ("job_creation_fee", "0"),
            ("job_last_updated_time", "1571797419")
        ])
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "terra1vladvladvladvladvladvladvladvladvl2000".to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {msgs: vec![
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: "cosmos2contract".to_string(),
                        amount: vec![coin(10000, "uluna")] })    
                ]})).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        })
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "terra1vladvladvladvladvladvladvladvladvl2000".to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {msgs: vec![
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: "vlad".to_string(),
                        amount: vec![coin(0, "uluna")] })    
                ]})).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        });

    assert_eq!(res, expected);
}

#[test]
fn test_create_job_success_multiple_msgs_all_types() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let cosmos_msgs: Vec<CosmosMsg> = vec![
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "contract".to_string(),
            msg: to_binary("test").unwrap(),
            funds: vec![coin(AMOUNT_TO_SEND, DENOM)],
        }),
        CosmosMsg::Bank(BankMsg::Send {
            to_address: "vlad2".to_string(),
            amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
        }),
        CosmosMsg::Gov(GovMsg::Vote {
            proposal_id: 0,
            vote: VoteOption::Yes,
        }),
        CosmosMsg::Staking(StakingMsg::Delegate {
            validator: "vladidator".to_string(),
            amount: coin(AMOUNT_TO_SEND, DENOM),
        }),
        CosmosMsg::Distribution(DistributionMsg::SetWithdrawAddress {
            address: "vladdress".to_string(),
        }),
        CosmosMsg::Ibc(IbcMsg::Transfer {
            channel_id: "channel_vlad".to_string(),
            to_address: "vlad3".to_string(),
            amount: coin(AMOUNT_TO_SEND, DENOM),
            timeout: IbcTimeout::with_block(IbcTimeoutBlock {
                revision: 0,
                height: 0,
            }),
        }),
        CosmosMsg::Stargate {
            type_url: "utl".to_string(),
            value: Default::default(),
        },
    ];

    let msgs = cosmos_msgs
        .iter()
        .map(|msg| serde_json_wasm::to_string(&msg).unwrap())
        .collect();

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "multiple jobs".to_string(),
        description: format!("multiple jobs").to_string(),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs,
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap();

    let expected = Response::new()
        .add_attributes(vec![
            ("action", "create_job"),
            ("job_id", "1"),
            ("job_owner", "terra1vladvladvladvladvladvladvladvladvl1000"),
            ("job_name", "multiple jobs"),
            ("job_status", "\"Pending\""),
            ("job_condition", "{\"expr\":{\"block_height\":{\"comparator\":\"42\",\"op\":\"gte\"}}}"),
            ("job_msgs", "[\"{\\\"wasm\\\":{\\\"execute\\\":{\\\"contract_addr\\\":\\\"contract\\\",\\\"msg\\\":\\\"InRlc3Qi\\\",\\\"funds\\\":[{\\\"denom\\\":\\\"uluna\\\",\\\"amount\\\":\\\"1000\\\"}]}}}\",\"{\\\"bank\\\":{\\\"send\\\":{\\\"to_address\\\":\\\"vlad2\\\",\\\"amount\\\":[{\\\"denom\\\":\\\"uluna\\\",\\\"amount\\\":\\\"1000\\\"}]}}}\",\"{\\\"gov\\\":{\\\"vote\\\":{\\\"proposal_id\\\":0,\\\"vote\\\":\\\"yes\\\"}}}\",\"{\\\"staking\\\":{\\\"delegate\\\":{\\\"validator\\\":\\\"vladidator\\\",\\\"amount\\\":{\\\"denom\\\":\\\"uluna\\\",\\\"amount\\\":\\\"1000\\\"}}}}\",\"{\\\"distribution\\\":{\\\"set_withdraw_address\\\":{\\\"address\\\":\\\"vladdress\\\"}}}\",\"{\\\"ibc\\\":{\\\"transfer\\\":{\\\"channel_id\\\":\\\"channel_vlad\\\",\\\"to_address\\\":\\\"vlad3\\\",\\\"amount\\\":{\\\"denom\\\":\\\"uluna\\\",\\\"amount\\\":\\\"1000\\\"},\\\"timeout\\\":{\\\"block\\\":{\\\"revision\\\":0,\\\"height\\\":0},\\\"timestamp\\\":null}}}}\",\"{\\\"stargate\\\":{\\\"type_url\\\":\\\"utl\\\",\\\"value\\\":\\\"\\\"}}\"]"),
            ("job_reward", "10000"),
            ("job_creation_fee", "0"),
            ("job_last_updated_time", "1571797419")
        ])
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "terra1vladvladvladvladvladvladvladvladvl2000".to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {msgs: vec![
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: "cosmos2contract".to_string(),
                        amount: vec![coin(10000, "uluna")] })
                ]})).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        })
        .add_submessage(SubMsg {
            id: 0,
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "terra1vladvladvladvladvladvladvladvladvl2000".to_string(),
                msg: to_binary(&account::ExecuteMsg::Generic(GenericMsg {msgs: vec![
                    CosmosMsg::Bank(BankMsg::Send {
                        to_address: "vlad".to_string(),
                        amount: vec![coin(0, "uluna")] })
                ]})).unwrap(),
                funds: vec![],
            }),
            gas_limit: None,
            reply_on: ReplyOn::Never,
        });

    assert_eq!(res, expected);
}

#[test]
fn test_create_job_no_account() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

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
        Uint128::new(0),
        Uint64::new(0),
        Uint64::new(0),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    )
    .unwrap();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "send funds job".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(res, ContractError::AccountDoesNotExist {});
}

#[test]
fn test_create_job_not_enough_funds_in_account() {
    const REWARD: u128 = ACCOUNT_BALANCE * 2;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "send funds job".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(res, ContractError::AccountBalanceSmallerThanJobReward {});
}

#[test]
fn test_create_job_reward_too_small() {
    const REWARD: u128 = MIN_REWARD - 1;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "send funds job".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(res, ContractError::RewardTooSmall {});
}

#[test]
fn test_create_job_invalid_condition() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let var = Variable::Static(StaticVariable {
        kind: VariableKind::Uint,
        name: "var1".to_string(),
        value: "var1 value".to_string(),
        update_fn: None,
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "send funds job".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![var],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(res, ContractError::InvalidVariables {});
}

#[test]
fn test_create_job_invalid_msgs() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let sg_msg: CosmosMsg = CosmosMsg::Stargate {
        type_url: "utl".to_string(),
        value: Default::default(),
    };

    let fake_msg = serde_json_wasm::to_string(&sg_msg)
        .unwrap()
        .replace("type_url", "type_urm");

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "send funds job".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![fake_msg],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(&res.to_string(), "Error deserializing data");
}

#[test]
fn test_create_job_name_too_short() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: "".to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(res, ContractError::NameTooShort {});
}

#[test]
fn test_create_job_name_too_long() {
    const REWARD: u128 = MIN_REWARD;
    const AMOUNT_TO_SEND: u128 = 1_000;
    const DENOM: &str = "uluna";
    const RECEIVER: &str = "vlad";
    const TARGET_BLOCK_HEIGHT: u64 = 42;
    const MAX_JOB_DESC_LENGTH: usize = 140;

    let (mut deps, env, mut info, account_address) = init_warp_account();

    let bank_msg: CosmosMsg<BankMsg> = CosmosMsg::Bank(BankMsg::Send {
        to_address: RECEIVER.to_string(),
        amount: vec![coin(AMOUNT_TO_SEND, DENOM)],
    });

    let create_job_msg = ExecuteMsg::CreateJob(CreateJobMsg {
        name: format!("{}", "q".repeat(MAX_JOB_DESC_LENGTH + 1)).to_string(),
        description: format!("send {AMOUNT_TO_SEND} {DENOM} to {RECEIVER}"),
        labels: vec![],
        condition: Condition::Expr(Box::new(Expr::BlockHeight(BlockExpr {
            comparator: Uint64::from(TARGET_BLOCK_HEIGHT),
            op: NumOp::Gte,
        }))),
        msgs: vec![serde_json_wasm::to_string(&bank_msg).unwrap()],
        vars: vec![],
        recurring: false,
        requeue_on_evict: false,
        reward: Uint128::from(REWARD),
        assets_to_withdraw: None,
    });

    info.sender = account_address;
    let res = execute(deps.as_mut(), env, info, create_job_msg).unwrap_err();

    assert_eq!(res, ContractError::NameTooLong {});
}
