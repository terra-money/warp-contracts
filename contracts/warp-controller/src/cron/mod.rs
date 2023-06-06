//! Crontab.rs is a library for parsing cron schedule expressions.

// #![deny(deprecated)]
// #![deny(missing_docs)]
// #![deny(unreachable_patterns)]
// #![deny(unused_extern_crates)]
// #![deny(unused_imports)]
// #![deny(unused_qualifications)]

// extern crate time;

// #[cfg(test)]
// #[macro_use(expect)]
// extern crate expectest;

// TODO: Get rid of these.
#[cfg(test)]
mod test_helpers;

mod crontab;
mod error;
mod parsing;
mod times;
pub mod tm;

// Exports
pub use crontab::Crontab;
pub use error::CrontabError;
pub use parsing::ScheduleComponents;
