use cosmwasm_std::Uint64;
use cw_orch::anyhow;
use cw_orch::prelude::*;
use interface::warp_controller::WarpController;
use interface::warp_funding_account::WarpFundingAccount;
use interface::warp_job_account::WarpJobAccount;
use interface::warp_job_account_tracker::WarpJobAccountTracker;
use interface::warp_legacy_account::WarpLegacyAccount;
use interface::warp_resolver::WarpResolver;
use interface::warp_templates::WarpTemplates;
use tokio::runtime::Runtime;
// We start by creating a runtime, which is required for a sync daemon.

pub fn deploy() -> anyhow::Result<()> {
    dotenv::dotenv().ok(); // Used to load the `.env` file if any
    pretty_env_logger::init(); // Used to log contract and chain interactions

    let rt = Runtime::new()?;
    let network = networks::PHOENIX_1;
    let chain = DaemonBuilder::default()
        .handle(rt.handle())
        .chain(network)
        .build()?;

    let funding_account = WarpFundingAccount::new("warp_funding_account", chain.clone());
    funding_account.upload()?;

    let job_account = WarpJobAccount::new("warp_job_account", chain.clone());
    job_account.upload()?;

    let job_account_tracker = WarpJobAccountTracker::new("warp_job_account_tracker", chain.clone());
    job_account_tracker.upload()?;
    job_account_tracker.instantiate(
        &job_account_tracker::InstantiateMsg {
            admin: chain.wallet().address()?.to_string(),
            warp_addr: "".to_string(),
        },
        Some(&Addr::unchecked(chain.wallet().address()?.to_string())),
        None,
    )?;

    let legacy_account = WarpLegacyAccount::new("warp_legacy_account", chain.clone());
    legacy_account.upload()?;

    let resolver = WarpResolver::new("warp_resolver", chain.clone());
    resolver.upload()?;
    resolver.instantiate(&resolver::InstantiateMsg {}, None, None)?;

    let templates = WarpTemplates::new("warp_templates", chain.clone());
    templates.upload()?;
    templates.instantiate(
        &templates::InstantiateMsg {
            owner: chain.wallet().address()?.to_string(),
            fee_denom: "uluna".to_string(),
            fee_collector: chain.wallet().address()?.to_string(),
            templates: vec![],
        },
        Some(&Addr::unchecked(chain.wallet().address()?.to_string())),
        None,
    )?;

    let controller = WarpController::new("warp_controller", chain.clone());

    controller.upload()?;
    controller.instantiate(
        &controller::InstantiateMsg {
            owner: Some(chain.wallet().address()?.to_string()),
            fee_denom: "".to_string(),
            fee_collector: Some(chain.wallet().address()?.to_string()),
            warp_account_code_id: Uint64::from(templates.code_id()?),
            minimum_reward: Default::default(),
            creation_fee: Default::default(),
            cancellation_fee: Default::default(),
            resolver_address: resolver.address()?.to_string(),
            job_account_tracker_address: job_account_tracker.address()?.to_string(),
            t_max: Default::default(),
            t_min: Default::default(),
            a_max: Default::default(),
            a_min: Default::default(),
            q_max: Default::default(),
            creation_fee_min: Default::default(),
            creation_fee_max: Default::default(),
            burn_fee_min: Default::default(),
            maintenance_fee_min: Default::default(),
            maintenance_fee_max: Default::default(),
            duration_days_left: Default::default(),
            duration_days_right: Default::default(),
            queue_size_left: Default::default(),
            queue_size_right: Default::default(),
            burn_fee_rate: Default::default(),
        },
        Some(&Addr::unchecked(chain.wallet().address()?.to_string())),
        None,
    )?;

    Ok(())
}
