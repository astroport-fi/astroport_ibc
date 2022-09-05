use cosmwasm_std::{Binary, StdError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Invalid reply id")]
    InvalidReplyId {},

    #[error("Unauthorized")]
    Unauthorized {},
}
