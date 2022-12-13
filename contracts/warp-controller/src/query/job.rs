use crate::state::{CONFIG, FINISHED_JOBS, PENDING_JOBS, QUERY_PAGE_SIZE};
use crate::util::filter::resolve_filters;
use cosmwasm_std::{Addr, Deps, Env, Order, StdError, StdResult, Uint64};
use cw_storage_plus::Bound;
use warp_protocol::controller::job::{
    JobResponse, JobStatus, JobsResponse, QueryJobMsg, QueryJobsMsg,
};

pub fn query_job(deps: Deps, _env: Env, data: QueryJobMsg) -> StdResult<JobResponse> {
    let job;
    if FINISHED_JOBS().has(deps.storage, data.id.u64()) {
        job = FINISHED_JOBS().load(deps.storage, data.id.u64())?
    } else {
        job = PENDING_JOBS().load(deps.storage, data.id.u64())?;
    }
    Ok(JobResponse { job })
}

pub fn query_jobs(deps: Deps, env: Env, data: QueryJobsMsg) -> StdResult<JobsResponse> {
    if !data.valid_query() {
        return Err(StdError::generic_err(
            "Invalid query input. Must supply at most one of ids, name, or owner params.",
        ));
    }

    let page_size = data.limit.unwrap_or(QUERY_PAGE_SIZE);

    match data {
        QueryJobsMsg {
            ids: Some(ids),
            job_status,
            condition_status,
            ..
        } => query_jobs_by_ids(deps, env, ids, job_status, condition_status),
        QueryJobsMsg {
            active: _,
            name,
            owner,
            job_status,
            condition_status,
            start_after,
            limit: _,
            ..
        } => query_jobs_by_reward(
            deps,
            env,
            name,
            owner,
            job_status,
            condition_status,
            match start_after {
                None => None,
                Some(i) => Some((i._0.u128(), i._1.u64())),
            },
            Some(page_size as usize),
        ),
    }
}

pub fn query_jobs_by_ids(
    deps: Deps,
    env: Env,
    ids: Vec<Uint64>,
    job_status: Option<JobStatus>,
    condition_status: Option<bool>,
) -> StdResult<JobsResponse> {
    if ids.len() > QUERY_PAGE_SIZE as usize {
        return Err(StdError::generic_err(
            "Number of ids supplied exceeds query limit",
        ));
    }

    let _config = CONFIG.load(deps.storage)?;
    let mut jobs = vec![];
    for id in ids {
        let query_msg = match job_status.clone() {
            None => QueryJobMsg { id },
            Some(_j) => QueryJobMsg { id },
        };

        let job = query_job(deps, env.clone(), query_msg)?.job;
        if resolve_filters(
            deps.clone(),
            env.clone(),
            job.clone(),
            None,
            None,
            condition_status.clone(),
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
    condition_status: Option<bool>,
    start_after: Option<(u128, u64)>,
    limit: Option<usize>,
) -> StdResult<JobsResponse> {
    let start = start_after.map(Bound::exclusive);
    if job_status.clone().is_some() && job_status.clone().unwrap() != JobStatus::Pending {
        let infos = FINISHED_JOBS()
            .idx
            .reward
            .range(deps.storage, None, start, Order::Descending)
            .filter(|h| {
                resolve_filters(
                    deps.clone(),
                    env.clone(),
                    h.as_ref().unwrap().clone().1,
                    name.clone(),
                    owner.clone(),
                    condition_status,
                    job_status.clone(),
                )
            });
        let infos = match limit {
            None => infos
                .take(QUERY_PAGE_SIZE as usize)
                .collect::<StdResult<Vec<_>>>()?,
            Some(limit) => infos.take(limit).collect::<StdResult<Vec<_>>>()?,
        };

        let mut jobs = vec![];
        for info in infos.clone() {
            jobs.push(info.1);
        }
        Ok(JobsResponse {
            jobs,
            total_count: infos.len(),
        })
    } else {
        let infos = PENDING_JOBS()
            .idx
            .reward
            .range(deps.storage, None, start, Order::Descending)
            .filter(|h| {
                resolve_filters(
                    deps.clone(),
                    env.clone(),
                    h.as_ref().unwrap().clone().1,
                    name.clone(),
                    owner.clone(),
                    condition_status,
                    job_status.clone(),
                )
            });
        let infos = match limit {
            None => infos
                .take(QUERY_PAGE_SIZE as usize)
                .collect::<StdResult<Vec<_>>>()?,
            Some(limit) => infos.take(limit).collect::<StdResult<Vec<_>>>()?,
        };

        let mut jobs = vec![];
        for info in infos.clone() {
            jobs.push(info.1);
        }
        Ok(JobsResponse {
            jobs,
            total_count: infos.len(),
        })
    }
}
