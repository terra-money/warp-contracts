use crate::util::condition::resolve_cond;
use cosmwasm_std::{Addr, Deps, Env};
use warp_protocol::controller::job::{Job, JobStatus};

pub fn resolve_filters(
    deps: Deps,
    env: Env,
    job: Job,
    name: Option<String>,
    owner: Option<Addr>,
    condition_status: Option<bool>,
    job_status: Option<JobStatus>,
) -> bool {
    //readability-optimized
    if job_status.is_some() && job_status.unwrap() != job.status {
        return false;
    }

    if name.is_some() && name.unwrap() != job.name {
        return false;
    }

    if owner.is_some() && owner.unwrap() != job.owner {
        return false;
    }
    if condition_status.is_some()
        && condition_status.unwrap()
            != resolve_cond(deps, env, job.condition).unwrap_or(!condition_status.unwrap())
    {
        return false;
    }

    return true;
}
