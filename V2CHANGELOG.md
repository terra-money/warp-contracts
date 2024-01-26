# V2 Changelog

## Major Updates

- **User’s Warp-Account Removed:** 
  - Replaced by job accounts.
  - `create_job` now creates a job account on the fly in the same transaction.
  - Users can provide funds in `info.funds` that are relayed to the job account.
  - Job account is used as a job session throughout job execution.
  - In case of recurring jobs, the same job account is kept through time.
    - Required for stateful jobs like trading strategies.

- **Multiple Funding Accounts:** 
  - Users can create and manage multiple funding accounts.
  - Funding account is used for distributing rewards to keepers and paying fees.
  - Otherwise, fees are subtracted from `info.funds`.
  - Useful for recurring jobs to provide fees and topups on the side.

- **New Fee Mechanism:** 
  - Introduces a more dynamic and flexible fee calculation system.
  - Creation Fee: Calculated based on the queue size. 
  - Maintenance Fee: Determined based on the duration in days.
  - Burn Fee: Computed from the job reward.
  - `total_fees = creation_fee + maintenance_fee + burn_fee`
  - `job cost = total_fees + reward`

- **New Contract Warp-Account-Tracker:** 
  - Used for management of job accounts and funding accounts.
  - Holds state for taken and free accounts by job_id.

## API Changes

### Removed
- `create_job.msgs`
- `create_job.condition`
- `create_job.requeue_on_evict`

### Added
- `create_job.executions`
  - Array of executions that operate like a switch.
  - Single execution contains msgs (warp msgs) and condition.
  - On job execution, the first execution condition that returns true top-down is taken.
- **Job Accounts with WarpMsg Struct:**
  - Job accounts now operate with WarpMsg struct.
  - Previously, warp accounts worked only with cosmos msgs.
  - WarpMsg:Generic is equivalent to a cosmos msg.
  - Added support for WarpMsg:WithdrawAssets and WarpMsg:IbcTransfer.
  - Extensible messaging standard within warp in case custom message formats are needed in the future.
- `create_job.operational_amount`
  - Without funding account: `operational_amount` needs to equal `total_fees + reward`.
  - With funding account: Ignored, can be set to 0.
- `create_job.duration_days`
  - Defines job length of stay in the warp queue.
  - Maintenance fee paid upfront for it based on fee calculations.
- `create_job.cw_funds`
  - Optionally passed list of cw20 and cw721 funds to be sent from user to job account.
- `create_job.funding_account`
  - Optionally attached funding account from which job fees and rewards are deducted.
  - Required for recurring jobs.
  - Optionally provided for one-time jobs.
- `create_job.account_msgs`
  - Messages that are executed via job-account on job creation.
  - Useful for deploying funds to money markets to earn APR while the job waits for execution.
- **Controller.create_funding_account API:**
  - Creates a new free funding account for the user.
- **FnValue StringValue Support:**
  - Static variable can be initialized with an `init_fn`.
  - FnValue now supports `StringValue`.

## Fee functions

### Creation Fee

The creation fee (`f(qs)`) is a piecewise function depending on the queue size (`qs`).

```
f(qs) = 
  y1, if qs < x1
  slope * qs + y1 - slope * x1, if x1 <= qs < x2
  y2, if qs >= x2
```

Where:
- `x1` = `config.queue_size_left`
- `x2` = `config.queue_size_right`
- `y1` = `config.creation_fee_min`
- `y2` = `config.creation_fee_max`
- `slope` = `(y2 - y1) / (x2 - x1)`

### Maintenance Fee

The maintenance fee (`g(dd)`) is structured similarly, based on the duration in days (`dd`).

```
g(dd) = 
  y1, if dd < x1
  slope * dd + y1 - slope * x1, if x1 <= dd < x2
  y2, if dd >= x2
```

Where:
- `x1` = `config.duration_days_min`
- `x2` = `config.duration_days_max`
- `slope` = `(y2 - y1) / (x2 - x1)`

### Burn Fee

The burn fee (`h(job_reward)`) is calculated as the maximum between the `calculated_fee` and `min_fee`.

```
h(job_reward) = 
  max(calculated_fee, min_fee)
```

Where:
- `calculated_fee` = `job_reward * config.burn_fee_rate / 100`
- `min_fee` = `config.burn_fee_min`

    