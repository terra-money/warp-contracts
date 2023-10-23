use controller::account::LegacyAccount;
use cosmwasm_std::Addr;

pub fn is_legacy_account(legacy_account: Option<LegacyAccount>, job_account_addr: Addr) -> bool {
    legacy_account.map_or(false, |legacy_account| {
        legacy_account.account == job_account_addr
    })
}
