use cosmwasm_std::{Binary, StdError};
use cw_utils::PaymentError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply id")]
    InvalidReplyId {},

    #[error("Channel already created {channel_id}")]
    ChannelAlreadyCreated { channel_id: String },

    #[error("Invalid source port {invalid}. Should be : {valid}")]
    InvalidSourcePort { invalid: String, valid: String },

    #[error("{0}")]
    PaymentError(#[from] PaymentError),
}
