use astroport_ibc::TIMEOUT_LIMITS;
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

    #[error("Proposal with id {proposal_id} was already executed.")]
    ProposalAlreadyExists { proposal_id: u64 },

    #[error(
        "Timeout must be within limits ({0} <= timeout <= {1})",
        TIMEOUT_LIMITS.start(),
        TIMEOUT_LIMITS.end()
    )]
    TimeoutLimitsError {},
}
