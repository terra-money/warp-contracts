use warp_controller_pkg::job::{Job, JobStatus};
use cosmwasm_std::{Addr, Deps, Env};

pub fn resolve_filters(
    _deps: Deps,
    _env: Env,
    job: Job,
    name: Option<String>,
    owner: Option<Addr>,
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

    true
}
