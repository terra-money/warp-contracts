use crate::ContractError::{CustomError, DecodeError, DeserializationError, SerializationError};
use cosmwasm_std::{OverflowError, StdError};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid fee")]
    InvalidFee {},

    #[error("Funds array in message does not match funds array in job.")]
    FundsMismatch {},

    #[error("Reward provided is smaller than minimum")]
    RewardTooSmall {},

    #[error("Invalid arguments")]
    InvalidArguments {},

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

    #[error("Template fee not found.")]
    TemplateFeeNotFound {},

    #[error("Template does not exist")]
    TemplateDoesNotExist {},

    #[error("Name must be at least 1 character long")]
    NameTooShort {},

    #[error("Name cannot exceed 280 characters")]
    NameTooLong {},
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

impl From<OverflowError> for ContractError {
    fn from(_: OverflowError) -> Self {
        CustomError {
            val: "ERROR: Overflow error".to_string(),
        }
    }
}
