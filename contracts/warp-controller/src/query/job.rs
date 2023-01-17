use crate::state::{CONFIG, FINISHED_JOBS, PENDING_JOBS, QUERY_PAGE_SIZE};
use crate::util::filter::resolve_filters;
use cosmwasm_std::{Addr, Deps, Env, Order, StdError, StdResult, Uint64};
use cw_storage_plus::Bound;
use warp_protocol::controller::job::{
    JobResponse, JobStatus, JobsResponse, QueryJobMsg, QueryJobsMsg,
};

pub fn query_job(deps: Deps, _env: Env, data: QueryJobMsg) -> StdResult<JobResponse> {
    let job = if FINISHED_JOBS().has(deps.storage, data.id.u64()) {
        FINISHED_JOBS().load(deps.storage, data.id.u64())?
    } else {
        PENDING_JOBS().load(deps.storage, data.id.u64())?
    };
    Ok(JobResponse { job })
}

pub fn query_jobs(deps: Deps, env: Env, data: QueryJobsMsg) -> StdResult<JobsResponse> {
    if !data.valid_query() {
        return Err(StdError::generic_err(
            "Invalid query input. Must supply at most one of ids, name, or owner params.",
        ));
    }

    let page_size = data.limit.unwrap_or(QUERY_PAGE_SIZE);

    if page_size > QUERY_PAGE_SIZE {
        return Err(StdError::generic_err(format!("Limit must be a max of {}.", QUERY_PAGE_SIZE)))
    }

    match data {
        QueryJobsMsg {
            ids: Some(ids),
            job_status,
            ..
        } => query_jobs_by_ids(deps, env, ids, job_status),
        QueryJobsMsg {
            active: _,
            name,
            owner,
            job_status,
            start_after,
            limit: _,
            ..
        } => query_jobs_by_reward(
            deps,
            env,
            name,
            owner,
            job_status,
            start_after.map(|i| (i._0.u128(), i._1.u64())),
            page_size as usize,
        ),
    }
}

pub fn query_jobs_by_ids(
    deps: Deps,
    env: Env,
    ids: Vec<Uint64>,
    job_status: Option<JobStatus>,
) -> StdResult<JobsResponse> {
    if ids.len() > QUERY_PAGE_SIZE as usize {
        return Err(StdError::generic_err(
            "Number of ids supplied exceeds query limit",
        ));
    }

    let mut jobs = vec![];
    for id in ids {
        let query_msg = QueryJobMsg { id };

        let job = query_job(deps, env.clone(), query_msg)?.job;
        if resolve_filters(
            deps,
            env.clone(),
            job.clone(),
            None,
            None,
            job_status.clone(),
        ) {
            jobs.push(job)
        }
    }
    Ok(JobsResponse {
        jobs: jobs.clone(),
        total_count: jobs.len(),
    })
}

pub fn query_jobs_by_reward(
    deps: Deps,
    env: Env,
    name: Option<String>,
    owner: Option<Addr>,
    job_status: Option<JobStatus>,
    start_after: Option<(u128, u64)>,
    limit: usize,
) -> StdResult<JobsResponse> {
    let start = start_after.map(Bound::exclusive);
    let map;
    if job_status.is_some() && job_status.clone().unwrap() != JobStatus::Pending {
        map = FINISHED_JOBS();
    } else {
        map = PENDING_JOBS();
    }
    let infos = map.idx
        .reward
        .range(deps.storage, None, start, Order::Descending)
        .filter(|h| {
            resolve_filters(
                deps,
                env.clone(),
                h.as_ref().unwrap().clone().1,
                name.clone(),
                owner.clone(),
                job_status.clone(),
            )
        })
        .take(limit).collect::<StdResult<Vec<_>>>()?;

    let mut jobs = vec![];
    for info in infos.clone() {
        jobs.push(info.1);
    }
    Ok(JobsResponse {
        jobs,
        total_count: infos.len(),
    })
}
