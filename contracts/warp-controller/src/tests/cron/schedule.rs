use crate::cron::Crontab;

#[test]
pub fn test_schedule_minute() {
    // https://terrasco.pe/mainnet/blocks/3859156
    // Monday, 20.2.2023, 08:14:59
    let time = 1676880899;
    let schedule = "* * * * *";
    let result = get_next(schedule, time);

    // one second is the next
    assert_eq!(result, 1676880900);

    let result = get_next(schedule, result);
    assert_eq!(result, 1676880900 + 60);
}

#[test]
pub fn test_schedule_monthly_weekday() {
    // https://terrasco.pe/mainnet/blocks/3859156
    // Monday, 20.2.2023, 08:14:59
    let time = 1676880899;

    // usually in crontab it would mean on day 1-7 of the month and also every monday.
    // we adjusted crontab to be more sane to combine both requirements, this means it executes on the first monday each month
    // if the other condition should be applied it is still possible in warp with multiple cron expressions and the expression-tree
    let schedule = "0 5 1-7 * 1";
    let result = get_next(schedule, time);

    // Monday, March 6, 2023 5:00:00 AM
    assert_eq!(result, 1678078800);

    let result = get_next(schedule, result);
    // Monday, April 3, 2023 5:00:00 AM
    assert_eq!(result, 1680498000);
}

fn get_next(schedule: &str, time: u64) -> u64 {
    Crontab::parse(schedule).unwrap().find_event_after(time)
}
