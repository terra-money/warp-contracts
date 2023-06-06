use super::tm::Tm;

/// Advance the year, but leave all other fields untouched.
/// This can result in an invalid day-of-month, day-of-year, or day-of-week!
pub(crate) fn adv_year(time: &mut Tm) {
    time.tm_year += 1;
}

/// Advance the day, but leave the day (and hour, minute, second) untouched.
/// This can result in an invalid day-of-month!
pub(crate) fn adv_month(time: &mut Tm) {
    time.tm_mon += 1;
    if time.tm_mon > 11 {
        time.tm_mon = 0;
        adv_year(time);
    }
}

/// Advance the day, but leave the hour, minute, and second untouched.
pub(crate) fn adv_day(time: &mut Tm) {
    time.tm_wday = (time.tm_wday + 1) % 7; // day of week
    time.tm_mday += 1; // day of month

    let is_leap_year = {
        let year = time.tm_year + 1900;
        if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) {
            true
        } else {
            false
        }
    };

    let days_in_year = if is_leap_year { 366 } else { 365 };

    time.tm_yday = (time.tm_yday + 1) % days_in_year; // day of year

    match time.tm_mon {
        0 | 2 | 4 | 6 | 7 | 9 | 11 => {
            if time.tm_mday > 31 {
                time.tm_mday = 1;
                adv_month(time);
            }
        }
        3 | 5 | 8 | 10 => {
            if time.tm_mday > 30 {
                time.tm_mday = 1;
                adv_month(time);
            }
        }
        1 => {
            let mdays = if is_leap_year { 29 } else { 28 };

            if time.tm_mday > mdays {
                time.tm_mday = 1;
                adv_month(time);
            }
        }
        _ => {} // bad user input
    }
}

/// Advance the hour, but leave the minute and second untouched.
pub(crate) fn adv_hour(time: &mut Tm) {
    time.tm_hour += 1;
    if time.tm_hour > 23 {
        time.tm_hour = 0;
        adv_day(time);
    }
}

/// Advance the minute, but leave the second untouched.
pub(crate) fn adv_minute(time: &mut Tm) {
    time.tm_min += 1;
    if time.tm_min > 59 {
        time.tm_min = 0;
        adv_hour(time);
    }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//   use expectest::prelude::*;
//   use test_helpers::get_tm;
//   use test_helpers::normal;
//   use time::{Timespec, at_utc};

//   #[test]
//   pub fn test_adv_year() {
//     let mut tm = get_tm(2017, 10, 6, 12, 24, 0);
//     adv_year(&mut tm);
//     expect!(normal(&tm)).to(be_equal_to(get_tm(2018, 10, 6, 12, 24, 0)));
//   }

//   #[test]
//   pub fn test_adv_month() {
//     // January
//     let mut tm = get_tm(2017, 1, 1, 12, 0, 0);
//     adv_month(&mut tm);
//     expect!(normal(&tm)).to(be_equal_to(get_tm(2017, 2, 1, 12, 0, 0)));

//     // December
//     let mut tm = get_tm(2017, 12, 1, 0, 0, 0);
//     adv_month(&mut tm);
//     expect!(normal(&tm)).to(be_equal_to(get_tm(2018, 1, 1, 0, 0, 0)));
//   }

//   #[test]
//   pub fn test_adv_day_on_mday() {
//     // 2017-01-01 00:00 UTC, a non-leap year starting on a Sunday (tm_wday=0).
//     let timespec = Timespec::new(1483228800, 0);
//     let mut tm = at_utc(timespec);

//     // 2017 to 2019 are not leap years
//     let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

//     // 2017 is (tm_year=117)
//     for tm_year in 117 .. 120 {
//       expect!(tm.tm_year).to(be_equal_to(tm_year));

//       for days_in_month in days_in_months.iter() {
//         let bound = days_in_month + 1; // 1-indexed
//         for expected_day in 1..bound {
//           expect!(tm.tm_mday).to(be_equal_to(expected_day));
//           adv_day(&mut tm);
//         }
//       }
//     }

//     expect!(tm.tm_year).to(be_equal_to(120));

//     // 2020 is a leap-year
//     let days_in_months = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

//     for days_in_month in days_in_months.iter() {
//       let bound = days_in_month + 1; // 1-indexed
//       for expected_day in 1..bound {
//         expect!(tm.tm_mday).to(be_equal_to(expected_day));
//         adv_day(&mut tm);
//       }
//     }

//     expect!(tm.tm_year).to(be_equal_to(121));
//   }

//   #[test]
//   pub fn test_adv_day_on_wday() {
//     // 2017-01-01 00:00 UTC, a non-leap year starting on a Sunday (tm_wday=0).
//     let timespec = Timespec::new(1483228800, 0);
//     let mut tm = at_utc(timespec);

//     // First week.
//     expect!(tm.tm_wday).to(be_equal_to(0));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(1));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(2));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(3));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(4));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(5));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(6));
//     adv_day(&mut tm);
//     expect!(tm.tm_wday).to(be_equal_to(0)); // Back to sunday!

//     // Four more years...
//     let mut expected = 0;
//     for _ in 0 .. 1460 {
//       expected = (expected + 1) % 7;
//       adv_day(&mut tm);
//       expect!(tm.tm_wday).to(be_equal_to(expected));
//     }

//     // Reset.
//     let mut tm = at_utc(timespec);

//     expect!(tm.tm_year).to(be_equal_to(117)); // 2017
//     expect!(tm.tm_wday).to(be_equal_to(0)); // Starts on a Sunday

//     // Entire year.
//     for _ in 0 .. 365 {
//       adv_day(&mut tm);
//     }

//     // Now it's 2018-01-01
//     expect!(tm.tm_year).to(be_equal_to(118)); // 2018
//     expect!(tm.tm_wday).to(be_equal_to(1)); // Starts on a Monday
//   }

//   #[test]
//   pub fn test_adv_day_on_yday() {
//     // 2017-01-01 00:00 UTC, a non-leap year starting on a Sunday (tm_wday=0).
//     let timespec = Timespec::new(1483228800, 0);
//     let mut tm = at_utc(timespec);

//     // First day of 2017. (tm_year=117)
//     expect!(tm.tm_year).to(be_equal_to(117));
//     expect!(tm.tm_yday).to(be_equal_to(0));

//     // 2017 passes...
//     for expected_day in 0 .. 365 {
//       expect!(tm.tm_year).to(be_equal_to(117)); // 2017
//       expect!(tm.tm_yday).to(be_equal_to(expected_day));
//       adv_day(&mut tm);
//     }

//     // First day of 2018.
//     expect!(tm.tm_year).to(be_equal_to(118));
//     expect!(tm.tm_yday).to(be_equal_to(0));

//     // 2018 and 2019 pass... (Also not leap years.)
//     for year in 118 .. 120 {
//       for expected_day in 0 .. 365 {
//         expect!(tm.tm_year).to(be_equal_to(year));
//         expect!(tm.tm_yday).to(be_equal_to(expected_day));
//         adv_day(&mut tm);
//       }
//     }

//     // First day of 2020.
//     expect!(tm.tm_year).to(be_equal_to(120));
//     expect!(tm.tm_yday).to(be_equal_to(0));

//     // This is a leap year!
//     for expected_day in 0 .. 366 {
//       expect!(tm.tm_year).to(be_equal_to(120));
//       expect!(tm.tm_yday).to(be_equal_to(expected_day));
//       adv_day(&mut tm);
//     }

//     // First day of 2021.
//     expect!(tm.tm_year).to(be_equal_to(121));
//     expect!(tm.tm_yday).to(be_equal_to(0));
//   }
// }
