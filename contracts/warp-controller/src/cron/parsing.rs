#![allow(deprecated)]

use super::error::CrontabError;
use std::collections::HashSet;
use std::iter::FromIterator;

/// The components of a crontab schedule.
/// The values in each field are guaranteed to be both unique and ordered.
#[derive(Clone, Debug, Default)]
pub struct ScheduleComponents {
    /// Minutes in the schedule.
    /// Range [0,59] inclusive.
    pub minutes: Vec<u32>,

    /// Hours in the schedule.
    /// Range [0,23] inclusive.
    pub hours: Vec<u32>,

    /// Days of the month in the schedule.
    /// Range [1,31] inclusive.
    pub days: Vec<u32>,

    /// Months in the schedule.
    /// Range [1,12] inclusive.
    pub months: Vec<u32>,

    /// Days of the week in the schedule.
    /// Range [0,6] inclusive.
    pub weekdays: Vec<u32>,
}

pub(crate) fn parse_cron(schedule: &str) -> Result<ScheduleComponents, CrontabError> {
    let fields: Vec<&str> = schedule.trim().split_whitespace().collect();

    if fields.len() != 5 {
        return Err(CrontabError::ErrCronFormat(format!(
            "Invalid format: {}",
            schedule
        )));
    }

    let minutes = parse_field(fields[0], 0, 59)?;
    let hours = parse_field(fields[1], 0, 23)?;
    let days = parse_field(fields[2], 1, 31)?;
    let months = parse_field(fields[3], 1, 12)?;
    let weekdays = parse_field(fields[4], 0, 6)?;

    Ok(ScheduleComponents {
        minutes: minutes,
        hours: hours,
        days: days,
        months: months,
        weekdays: weekdays,
    })
}

fn parse_field(field: &str, field_min: u32, field_max: u32) -> Result<Vec<u32>, CrontabError> {
    let mut components = HashSet::<u32>::new();

    for part in field.split(",") {
        let mut min = field_min;
        let mut max = field_max;
        let mut step = 1;

        // stepped, eg. */2 or 1-45/3
        let stepped: Vec<&str> = part.splitn(2, "/").collect();

        // ranges, eg. 1-30
        let range: Vec<&str> = stepped[0].splitn(2, "-").collect();

        if stepped.len() == 2 {
            step = stepped[1].parse::<u32>()?;
        }

        if range.len() == 2 {
            min = range[0].parse::<u32>()?;
            max = range[1].parse::<u32>()?;
        }

        if stepped.len() == 1 && range.len() == 1 && part != "*" {
            min = part.parse::<u32>()?;
            max = min;
        }

        if min < field_min {
            return Err(CrontabError::FieldOutsideRange {
                description: format!("Value {} is less than minimum: {}", min, field_min),
            });
        }

        if max > field_max {
            return Err(CrontabError::FieldOutsideRange {
                description: format!("Value {} is greater than maximum: {}", max, field_max),
            });
        }

        let values = (min..max + 1)
            .filter(|i| i % step == 0)
            .collect::<Vec<u32>>();

        components.extend(values);
    }

    let mut components: Vec<u32> = Vec::from_iter(components.into_iter());
    components.sort();

    Ok(components)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use expectest::prelude::*;

//     #[test]
//     fn parse_fields() {
//         // Precise number of fields
//         expect!(parse_cron("* * * * *")).to(be_ok());
//         // Incorrect number of fields
//         expect!(parse_cron("")).to(be_err());
//         expect!(parse_cron("* * * *")).to(be_err());
//         expect!(parse_cron("* * * * * *")).to(be_err());
//     }

//     #[test]
//     fn parse_whitespace() {
//         // Allowed whitespace
//         expect!(parse_cron("  * * * * *  ")).to(be_ok());
//         expect!(parse_cron("\n\t* * * * *\n\t")).to(be_ok());
//         expect!(parse_cron("\n\t*\n\t*\n\t*\n\t*\n\t*\n\t")).to(be_ok());
//         expect!(parse_cron("    *    *    *    *    *    ")).to(be_ok());
//         expect!(parse_cron("\r\r*\r\n*\t\t*\t\t*\n  *\n\r")).to(be_ok());
//     }

//     #[test]
//     fn wildcards() {
//         let parsed = parse_cron("* * * * *").unwrap();

//         expect!(parsed.minutes).to(be_equal_to((0..60).collect::<Vec<u32>>()));
//         expect!(parsed.hours).to(be_equal_to((0..24).collect::<Vec<u32>>()));
//         expect!(parsed.days).to(be_equal_to((1..32).collect::<Vec<u32>>()));
//         expect!(parsed.months).to(be_equal_to((1..13).collect::<Vec<u32>>()));
//         expect!(parsed.weekdays).to(be_equal_to((0..7).collect::<Vec<u32>>()));
//     }

//     #[test]
//     fn ranges() {
//         let parsed = parse_cron("0-5 20-23 1-5 1-6 0-6").unwrap();

//         expect!(parsed.minutes).to(be_equal_to(vec![0, 1, 2, 3, 4, 5]));
//         expect!(parsed.hours).to(be_equal_to(vec![20, 21, 22, 23]));
//         expect!(parsed.days).to(be_equal_to(vec![1, 2, 3, 4, 5]));
//         expect!(parsed.months).to(be_equal_to(vec![1, 2, 3, 4, 5, 6]));
//         expect!(parsed.weekdays).to(be_equal_to(vec![0, 1, 2, 3, 4, 5, 6]));
//     }

//     #[test]
//     fn step() {
//         let parsed = parse_cron("*/15 */4 */10 */3 */2").unwrap();

//         expect!(parsed.minutes).to(be_equal_to(vec![0, 15, 30, 45]));
//         expect!(parsed.hours).to(be_equal_to(vec![0, 4, 8, 12, 16, 20]));
//         expect!(parsed.days).to(be_equal_to(vec![10, 20, 30]));
//         expect!(parsed.months).to(be_equal_to(vec![3, 6, 9, 12]));
//         expect!(parsed.weekdays).to(be_equal_to(vec![0, 2, 4, 6]));
//     }

//     #[test]
//     fn ranges_with_step() {
//         let parsed = parse_cron("0-30/5 0-12/2 1-20/5 1-10/2 0-5/2").unwrap();

//         assert_eq!(parsed.minutes, vec![0, 5, 10, 15, 20, 25, 30]);
//         expect!(parsed.hours).to(be_equal_to(vec![0, 2, 4, 6, 8, 10, 12]));
//         expect!(parsed.days).to(be_equal_to(vec![5, 10, 15, 20]));
//         expect!(parsed.months).to(be_equal_to(vec![2, 4, 6, 8, 10]));
//         expect!(parsed.weekdays).to(be_equal_to(vec![0, 2, 4]));
//     }

//     #[test]
//     fn comma_separated() {
//         let parsed = parse_cron("0,5,15 0,12 1,15 1,3,6,9,12 0,1,2,3,4").unwrap();

//         expect!(parsed.minutes).to(be_equal_to(vec![0, 5, 15]));
//         expect!(parsed.hours).to(be_equal_to(vec![0, 12]));
//         expect!(parsed.days).to(be_equal_to(vec![1, 15]));
//         expect!(parsed.months).to(be_equal_to(vec![1, 3, 6, 9, 12]));
//         expect!(parsed.weekdays).to(be_equal_to(vec![0, 1, 2, 3, 4]));
//     }

//     #[test]
//     fn exact_minutes() {
//         let parsed = parse_cron("0 * * * *").unwrap();
//         expect!(parsed.minutes).to(be_equal_to(vec![0]));

//         let parsed = parse_cron("5,10,15 * * * *").unwrap();
//         expect!(parsed.minutes).to(be_equal_to(vec![5, 10, 15]));

//         let parsed = parse_cron("59 * * * *").unwrap();
//         expect!(parsed.minutes).to(be_equal_to(vec![59]));
//     }

//     #[test]
//     fn exact_hours() {
//         let parsed = parse_cron("* 0 * * *").unwrap();
//         expect!(parsed.hours).to(be_equal_to(vec![0]));

//         let parsed = parse_cron("* 1,12,20 * * *").unwrap();
//         expect!(parsed.hours).to(be_equal_to(vec![1, 12, 20]));

//         let parsed = parse_cron("* 23 * * *").unwrap();
//         expect!(parsed.hours).to(be_equal_to(vec![23]));
//     }

//     #[test]
//     fn exact_days() {
//         let parsed = parse_cron("* * 1 * *").unwrap();
//         expect!(parsed.days).to(be_equal_to(vec![1]));

//         let parsed = parse_cron("* * 1,10,20,30 * *").unwrap();
//         expect!(parsed.days).to(be_equal_to(vec![1, 10, 20, 30]));

//         let parsed = parse_cron("* * 31 * *").unwrap();
//         expect!(parsed.days).to(be_equal_to(vec![31]));
//     }

//     #[test]
//     fn exact_months() {
//         let parsed = parse_cron("* * * 1 *").unwrap();
//         expect!(parsed.months).to(be_equal_to(vec![1]));

//         let parsed = parse_cron("* * * 1,5,7,10 *").unwrap();
//         expect!(parsed.months).to(be_equal_to(vec![1, 5, 7, 10]));

//         let parsed = parse_cron("* * * 12 *").unwrap();
//         expect!(parsed.months).to(be_equal_to(vec![12]));
//     }

//     #[test]
//     fn exact_weekdays() {
//         let parsed = parse_cron("* * * * 0").unwrap();
//         expect!(parsed.weekdays).to(be_equal_to(vec![0]));

//         let parsed = parse_cron("* * * * 1,2").unwrap();
//         expect!(parsed.weekdays).to(be_equal_to(vec![1, 2]));

//         let parsed = parse_cron("* * * * 6").unwrap();
//         expect!(parsed.weekdays).to(be_equal_to(vec![6]));
//     }

//     #[test]
//     fn exact_values_outside_range() {
//         // Minutes
//         expect!(parse_cron("60 * * * *")).to(be_err());
//         expect!(parse_cron("-1 * * * *")).to(be_err());
//         // Hours
//         expect!(parse_cron("* 24 * * *")).to(be_err());
//         expect!(parse_cron("* -1 * * *")).to(be_err());
//         // Days
//         expect!(parse_cron("* * 0 * *")).to(be_err());
//         expect!(parse_cron("* * 32 * *")).to(be_err());
//         expect!(parse_cron("* * -1 * *")).to(be_err());
//         // Months
//         expect!(parse_cron("* * * 0 *")).to(be_err());
//         expect!(parse_cron("* * * 13 *")).to(be_err());
//         expect!(parse_cron("* * * -1 *")).to(be_err());
//         // Weekdays
//         expect!(parse_cron("* * * * 7")).to(be_err());
//         expect!(parse_cron("* * * * -1")).to(be_err());
//     }

//     #[test]
//     fn ranges_outside_range() {
//         // Minutes
//         expect!(parse_cron("0-60 * * * *")).to(be_err());
//         expect!(parse_cron("-1-0 * * * *")).to(be_err());
//         // Hours
//         expect!(parse_cron("* 0-24 * * *")).to(be_err());
//         expect!(parse_cron("* -1-10 * * *")).to(be_err());
//         // Days
//         expect!(parse_cron("* * 0-5 * *")).to(be_err());
//         expect!(parse_cron("* * 10-32 * *")).to(be_err());
//         expect!(parse_cron("* * -1-20 * *")).to(be_err());
//         // Months
//         expect!(parse_cron("* * * 0-5 *")).to(be_err());
//         expect!(parse_cron("* * * 6-13 *")).to(be_err());
//         expect!(parse_cron("* * * -1-12 *")).to(be_err());
//         // Weekdays
//         expect!(parse_cron("* * * * 5-7")).to(be_err());
//         expect!(parse_cron("* * * * -1-5")).to(be_err());
//     }

//     #[test]
//     fn values_deduped() {
//         let parsed = parse_cron("1,1,1,1 * * * *").unwrap();
//         expect!(parsed.minutes).to(be_equal_to(vec![1]));

//         let parsed = parse_cron("1,1-3 * * * *").unwrap();
//         expect!(parsed.minutes).to(be_equal_to(vec![1, 2, 3]));

//         let parsed = parse_cron("* * * 1-4,2,4,*/2 *").unwrap();
//         expect!(parsed.months).to(be_equal_to(vec![1, 2, 3, 4, 6, 8, 10, 12]));
//     }

//     #[test]
//     fn values_in_order() {
//         let parsed = parse_cron("4,3,1,2 * * * *").unwrap();
//         expect!(parsed.minutes).to(be_equal_to(vec![1, 2, 3, 4]));
//     }

//     #[test]
//     fn misc_parse_errors() {
//         // Invalid values
//         expect!(parse_cron("A B C D E")).to(be_err());
//         expect!(parse_cron("*A *B *C *D *E")).to(be_err());

//         // No numeric values
//         expect!(parse_cron(", * * * *")).to(be_err());
//         expect!(parse_cron(",,,, * * * *")).to(be_err());

//         // No range
//         expect!(parse_cron("- * * * *")).to(be_err());
//         expect!(parse_cron(",- * * * *")).to(be_err());
//         expect!(parse_cron("-,- * * * *")).to(be_err());
//         expect!(parse_cron(",-,- * * * *")).to(be_err());
//         expect!(parse_cron(",-,-, * * * *")).to(be_err());

//         // Allowed whitespace, but incorrect number of fields
//         expect!(parse_cron("   ")).to(be_err());
//         expect!(parse_cron("  * * * *  ")).to(be_err());
//         expect!(parse_cron("  * * * * * *  ")).to(be_err());
//         expect!(parse_cron("\n\t")).to(be_err());
//         expect!(parse_cron("\n\t* * * *\n\t")).to(be_err());
//         expect!(parse_cron("\n\t* * * * * *\n\t")).to(be_err());
//     }
// }
