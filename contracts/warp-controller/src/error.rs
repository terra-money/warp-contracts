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

    #[error("Name cannot exceed 280 characters")]
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

    #[error("Hydration error: {msg:?}")]
    HydrationError { msg: String },

    #[error("Function error: {msg:?}")]
    FunctionError { msg: String },

    #[error("Variable not found: {name:?}.")]
    VariableNotFound { name: String },

    #[error("Condition error: {msg:?}")]
    ConditionError { msg: String },

    #[error("Msg error: {msg:?}")]
    MsgError { msg: String },

    #[error("Max eviction fee smaller than minimum eviction fee.")]
    MaxFeeUnderMinFee {},

    #[error("Max eviction time smaller than minimum eviction time.")]
    MaxTimeUnderMinTime {},

    #[error("Job reward smaller than eviction fee.")]
    RewardSmallerThanFee {},

    #[error("Invalid variables.")]
    InvalidVariables {},

    #[error("Variables list contains duplicates.")]
    VariablesContainDuplicates {},

    #[error("Eviction period not elapsed.")]
    EvictionPeriodNotElapsed {},

    #[error("Variables in condition or msgs missing from variables vector.")]
    VariablesMissingFromVector {},

    #[error("Variable vector contains unused variables.")]
    ExcessVariablesInVector {},
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
            val: "ERROR: Overflow error".to_string(),
        }
    }
}

pub fn map_contract_error(e: &str) -> String {
    if e.contains("wasm") {
        if e.contains("code: 28") {
            "No such code ID."
        } else if e.contains("code: 27") {
            "Max query stack size exceeded."
        } else if e.contains("code: 22") {
            "No such contract at requested address."
        } else if e.contains("code: 21") {
            "Invalid event from contract."
        } else if e.contains("code: 20") {
            "Unknown message from the contract."
        } else if e.contains("code: 19") {
            "Unpinning contract failed."
        } else if e.contains("code: 18") {
            "Pinning contract failed."
        } else if e.contains("code: 17") {
            "Unsupported action for this contract."
        } else if e.contains("code: 16") {
            "Maximum IBC channels reached."
        } else if e.contains("code: 15") {
            "Content is duplicated."
        } else if e.contains("code: 14") {
            "Content is invalid in this context."
        } else if e.contains("code: 13") {
            "Content exceeds limit."
        } else if e.contains("code: 12") {
            "Empty content."
        } else if e.contains("code: 11") {
            "Migrate wasm contract failed."
        } else if e.contains("code: 10") {
            "Invalid CosmosMsg from the called contract."
        } else if e.contains("code: 9") {
            "Query wasm contract failed."
        } else if e.contains("code: 8") {
            "Entry not found in store."
        } else if e.contains("code: 7") {
            "Invalid genesis file."
        } else if e.contains("code: 6") {
            "Insufficient gas."
        } else if e.contains("code: 5") {
            "Execute wasm contract failed. Common causes include insufficient CW20 Funds, permission errors on CW721 assets, and malformed contract messages."
        } else if e.contains("code: 4") {
            "Instantiate wasm contract failed."
        } else if e.contains("code: 3") {
            "Contract account already exists."
        } else if e.contains("code: 2") {
            "Create wasm contract failed."
        } else {
            "Undefined error."
        }
    } else if e.contains("sdk") {
        if e.contains("code: 41") {
            "Invalid gas limit."
        } else if e.contains("code: 40") {
            "Error in app.toml."
        } else if e.contains("code: 39") {
            "Internal IO error."
        } else if e.contains("code: 38") {
            "Not found: Entity does not exist in state."
        } else if e.contains("code: 37") {
            "Feature not supported."
        } else if e.contains("code: 36") {
            "Conflict error."
        } else if e.contains("code: 35") {
            "Internal logic error."
        } else if e.contains("code: 34") {
            "Failed unpacking protobuf msg."
        } else if e.contains("code: 33") {
            "Failed packing protobuf msg."
        } else if e.contains("code: 32") {
            "Incorrect account sequence."
        } else if e.contains("code: 31") {
            "Unknown extension options."
        } else if e.contains("code: 30") {
            "Tx timeout height."
        } else if e.contains("code: 29") {
            "Invalid type."
        } else if e.contains("code: 28") {
            "Invalid chain-id."
        } else if e.contains("code: 27") {
            "Invalid version."
        } else if e.contains("code: 26") {
            "invalid height."
        } else if e.contains("code: 25") {
            "Invalid gas adjustment."
        } else if e.contains("code: 24") {
            "Tx indended signer does not match the given signer."
        } else if e.contains("code: 23") {
            "Invalid account password."
        } else if e.contains("code: 22") {
            "Key not found."
        } else if e.contains("code: 21") {
            "Tx too large."
        } else if e.contains("code: 20") {
            "Mempool is full."
        } else if e.contains("code: 19") {
            "Tx already in mempool."
        } else if e.contains("code: 18") {
            "Invalid request."
        } else if e.contains("code: 17") {
            "Failed to unmarshal JSON bytes."
        } else if e.contains("code: 16") {
            "Failed to marshal JSON bytes."
        } else if e.contains("code: 15") {
            "No signatures supplied."
        } else if e.contains("code: 14") {
            "Maximum number of signatures exceeded."
        } else if e.contains("code: 13") {
            "Insufficient fee."
        } else if e.contains("code: 12") {
            "Memo too large."
        } else if e.contains("code: 11") {
            "Out of gas."
        } else if e.contains("code: 10") {
            "Invalid coins."
        } else if e.contains("code: 9") {
            "Unknown address."
        } else if e.contains("code: 8") {
            "Invalid pubkey."
        } else if e.contains("code: 7") {
            "Invalid address."
        } else if e.contains("code: 6") {
            "Unknown request."
        } else if e.contains("code: 5") {
            "Invalid funds. Ensure that sufficient native tokens are being supplied for the job."
        } else if e.contains("code: 4") {
            "Unauthorized SDK request."
        } else if e.contains("code: 3") {
            "Invalid sequence."
        } else if e.contains("code: 2") {
            "Tx parse error."
        } else {
            "Undefined error."
        }
    } else {
        "Undefined error."
    }.to_string()
}
