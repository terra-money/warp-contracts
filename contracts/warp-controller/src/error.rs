use crate::ContractError::{CustomError, DecodeError, DeserializationError, SerializationError};
use cosmwasm_std::{DivideByZeroError, OverflowError, StdError};
use std::num::ParseIntError;
use std::str::ParseBoolError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Funds array in message does not match funds array in job.")]
    FundsMismatch {},

    #[error("Reward provided is smaller than minimum")]
    RewardTooSmall {},

    #[error("Name must be at least 1 character long")]
    NameTooShort {},

    #[error("Name cannot exceed 140 characters")]
    NameTooLong {},

    #[error("Attempting to distribute more rewards than received from the action")]
    DistributingMoreRewardThanReceived {},

    #[error("Invalid arguments")]
    InvalidArguments {},

    #[error("Account does not exist")]
    AccountDoesNotExist {},

    #[error("Account already exists")]
    AccountAlreadyExists {},

    #[error("Account cannot create an account")]
    AccountCannotCreateAccount {},

    #[error("Job already finished")]
    JobAlreadyFinished {},

    #[error("Job already exists")]
    JobAlreadyExists {},

    #[error("Job does not exist")]
    JobDoesNotExist {},

    #[error("Job not active")]
    JobNotActive {},

    #[error("Cancellation fee too high")]
    CancellationFeeTooHigh {},

    #[error("Creation fee too high")]
    CreationFeeTooHigh {},

    #[error("Template does not exist")]
    TemplateDoesNotExist {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Error deserializing data")]
    DeserializationError {},

    #[error("Error serializing data")]
    SerializationError {},

    #[error("Error decoding JSON result")]
    DecodeError {},

    #[error("Error resolving JSON path")]
    ResolveError {},
}

impl From<serde_json_wasm::de::Error> for ContractError {
    fn from(_: serde_json_wasm::de::Error) -> Self {
        DeserializationError {}
    }
}

impl From<serde_json_wasm::ser::Error> for ContractError {
    fn from(_: serde_json_wasm::ser::Error) -> Self {
        SerializationError {}
    }
}

impl From<json_codec_wasm::DecodeError> for ContractError {
    fn from(_: json_codec_wasm::DecodeError) -> Self {
        DecodeError {}
    }
}

impl From<base64::DecodeError> for ContractError {
    fn from(_: base64::DecodeError) -> Self {
        DecodeError {}
    }
}

impl From<String> for ContractError {
    fn from(val: String) -> Self {
        CustomError { val }
    }
}

impl From<ParseIntError> for ContractError {
    fn from(_: ParseIntError) -> Self {
        CustomError {
            val: "Parse int error".to_string(),
        }
    }
}

impl From<ParseBoolError> for ContractError {
    fn from(_: ParseBoolError) -> Self {
        CustomError {
            val: "Parse bool error".to_string(),
        }
    }
}

impl From<DivideByZeroError> for ContractError {
    fn from(_: DivideByZeroError) -> Self {
        CustomError {
            val: "ERROR: Division by zero".to_string(),
        }
    }
}

impl From<OverflowError> for ContractError {
    fn from(_: OverflowError) -> Self {
        CustomError {
            val: "ERROR: Overflow error".to_string()
        }
    }
}
