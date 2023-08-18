use astroport_ibc::{SIGNAL_OUTAGE_LIMITS, TIMEOUT_LIMITS};
use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

/// Never is a placeholder to ensure we don't return any errors
#[derive(Error, Debug)]
pub enum Never {}

#[derive(Debug, Error, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Contract can't be migrated!")]
    MigrationError {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid reply id")]
    InvalidReplyId {},

    #[error("Channel already established: {channel_id}")]
    ChannelAlreadyEstablished { channel_id: String },

    #[error("Invalid governance channel: {invalid}. Should be {valid}")]
    InvalidGovernanceChannel { invalid: String, valid: String },

    #[error("Governance is not established yet")]
    GovernanceChannelNotFound {},

    #[error("Invalid source port {invalid}. Should be : {valid}")]
    InvalidSourcePort { invalid: String, valid: String },

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("Messages check passed. Nothing was committed to the blockchain")]
    MessagesCheckPassed {},

    #[error("The gov_channel and the accept_new_connections settings cannot be specified at the same time")]
    UpdateChannelError {},

    #[error(
        "Timeout must be within limits ({0} <= timeout <= {1})",
        TIMEOUT_LIMITS.start(),
        TIMEOUT_LIMITS.end()
    )]
    TimeoutLimitsError {},
    #[error(
        "Signal outage must be within limits ({0} <= outage <= {1})",
        SIGNAL_OUTAGE_LIMITS.start(),
        SIGNAL_OUTAGE_LIMITS.end()
    )]
    SignalOutageLimitsError {},
}
