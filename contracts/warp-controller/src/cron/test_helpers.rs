// //! Functions used in the tests.
// //! This module is only compiled for testing.

// use expectest::prelude::*;
// use time::Tm;

// // TODO: Get rid of this. Not really necessary. Just hardcode dates and times.

// /// Get a Tm from a date. Months and days are supplied 1-indexed, but
// /// the Tm struct is inconsistently 0- and 1-indexed.
// pub (crate) fn get_tm(year: i32,
//                       month: i32,
//                       day: i32,
//                       hour: i32,
//                       minute: i32,
//                       second: i32) -> Tm {

//   expect!(month).to(be_greater_or_equal_to(1));
//   expect!(month).to(be_less_or_equal_to(12));
//   expect!(day).to(be_greater_or_equal_to(1));
//   expect!(day).to(be_less_or_equal_to(31));
//   expect!(hour).to(be_greater_or_equal_to(0));
//   expect!(hour).to(be_less_than(24));
//   expect!(minute).to(be_greater_or_equal_to(0));
//   expect!(minute).to(be_less_than(60));
//   expect!(second).to(be_greater_or_equal_to(0));
//   expect!(second).to(be_less_or_equal_to(60)); // leap seconds

//   Tm {
//     tm_sec: second,
//     tm_min: minute,
//     tm_hour: hour,
//     tm_mday: day,
//     tm_mon: month.saturating_sub(1), // zero indexed
//     tm_year: year.saturating_sub(1900), // Years since 1900
//     tm_wday: 0, // Incorrect, but don't care
//     tm_yday: 0, // Incorrect, but don't care
//     tm_isdst: 0,
//     tm_utcoff: 0,
//     tm_nsec: 0,
//   }
// }

// /// Normalize a Tm to drop certain fields entirely.
// pub (crate) fn normal(time: &Tm) -> Tm {
//   let mut tm = time.clone();
//   tm.tm_wday = 0;
//   tm.tm_yday = 0;
//   tm.tm_isdst = 0;
//   tm.tm_utcoff = 0;
//   tm.tm_nsec= 0;
//   tm
// }
