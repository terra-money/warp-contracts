use cosmwasm_std::Addr;

pub fn is_sub_account(main_account_addr: &Addr, job_account_addr: &Addr) -> bool {
    main_account_addr != job_account_addr
}
