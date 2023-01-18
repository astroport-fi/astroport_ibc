use crate::contract::MIN_TIMEOUT;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Contract can't be migrated!")]
    MigrationError {},

    #[error("A proposal with this ID already exists: {proposal_id}")]
    ProposalAlreadyExists { proposal_id: u64 },

    #[error(
        "Timeout must be within limits ({0} < timeout <= {1})",
        MIN_TIMEOUT,
        u64::MAX
    )]
    TimeoutLimitsError {},
}
