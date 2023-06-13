pub mod beta {
    use controller::condition::Condition;
    use controller::job::JobStatus;
    use controller::variable::{Method, QueryVariable, StaticVariable, UpdateFn, VariableKind};
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, Uint128, Uint64};
    use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex, UniqueIndex};

    // #[cw_serde]
    // pub struct State {
    //     pub current_job_id: Uint64,
    //     pub current_template_id: Uint64,
    //     pub q: Uint64,
    // }
    //
    // pub const STATE: Item<State> = Item::new("state");

    #[cw_serde]
    pub struct Config {
        pub owner: Addr,
        pub fee_collector: Addr,
        pub warp_account_code_id: Uint64,
        pub minimum_reward: Uint128,
        pub creation_fee_percentage: Uint64,
        pub cancellation_fee_percentage: Uint64,
        pub template_fee: Uint128,
        pub t_max: Uint64,
        pub t_min: Uint64,
        pub a_max: Uint128,
        pub a_min: Uint128,
        pub q_max: Uint64,
    }

    #[cw_serde]
    pub struct Job {
        pub id: Uint64,
        pub owner: Addr,
        pub last_update_time: Uint64,
        pub name: String,
        pub status: JobStatus,
        pub condition: Condition,
        pub msgs: Vec<String>,
        pub vars: Vec<Variable>,
        pub recurring: bool,
        pub requeue_on_evict: bool,
        pub reward: Uint128,
    }

    #[cw_serde]
    pub enum Variable {
        Static(StaticVariable),
        External(ExternalVariable),
        Query(QueryVariable),
    }

    #[cw_serde]
    pub struct ExternalVariable {
        pub kind: VariableKind,
        pub name: String,
        pub init_fn: ExternalExpr,
        pub reinitialize: bool,
        pub value: Option<String>, //none if uninitialized
        pub update_fn: Option<UpdateFn>,
    }

    #[cw_serde]
    pub struct ExternalExpr {
        pub url: String,
        pub method: Option<Method>,
        pub body: Option<String>,
        pub selector: String,
    }

    pub struct JobIndexes<'a> {
        pub reward: UniqueIndex<'a, (u128, u64), Job>,
        pub publish_time: MultiIndex<'a, u64, Job, u64>,
    }

    impl IndexList<Job> for JobIndexes<'_> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Job>> + '_> {
            let v: Vec<&dyn Index<Job>> = vec![&self.reward, &self.publish_time];
            Box::new(v.into_iter())
        }
    }

    #[allow(non_snake_case)]
    pub fn PENDING_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
        let indexes = JobIndexes {
            reward: UniqueIndex::new(
                |job| (job.reward.u128(), job.id.u64()),
                "pending_jobs__reward",
            ),
            publish_time: MultiIndex::new(
                |_pk, job| job.last_update_time.u64(),
                "pending_jobs",
                "pending_jobs__publish_timestamp",
            ),
        };
        IndexedMap::new("pending_jobs", indexes)
    }

    #[allow(non_snake_case)]
    pub fn FINISHED_JOBS<'a>() -> IndexedMap<'a, u64, Job, JobIndexes<'a>> {
        let indexes = JobIndexes {
            reward: UniqueIndex::new(
                |job| (job.reward.u128(), job.id.u64()),
                "finished_jobs__reward",
            ),
            publish_time: MultiIndex::new(
                |_pk, job| job.last_update_time.u64(),
                "finished_jobs",
                "finished_jobs__publish_timestamp",
            ),
        };
        IndexedMap::new("finished_jobs", indexes)
    }
}
