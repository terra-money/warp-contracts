use std::num::ParseIntError;
use thiserror::Error;

// TODO: These errors could use some improvement, but that would be breaking.
/// A library error.

#[derive(Error, Debug, PartialEq)]
pub enum CrontabError {
    /// Error parsing the crontab schedule.
    #[error("Error parsing cron format: {0}")]
    ErrCronFormat(String),

    /// Error parsing an integer in a crontab schedule.
    #[error("Error parsing int: {0}")]
    ErrParseInt(#[from] ParseIntError),

    /// Parse error. When one of the cron schedule fields is outside of the
    /// permitted range.
    #[error("One of the fields is outside the permitted range: {description}")]
    FieldOutsideRange {
        /// Description of the error.
        description: String,
    },
}
