use astroport_governance::assembly::{ProposalMessage, ProposalStatus};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub assembly: String,
    pub timeout: u64,
}

#[cw_serde]
pub struct IbcProposal {
    pub id: u64,
    pub messages: Vec<ProposalMessage>,
}

#[cw_serde]
pub enum ExecuteMsg {
    IbcExecuteProposal {
        channel_id: String,
        proposal_id: u64,
        messages: Vec<ProposalMessage>,
    },
    UpdateConfig {
        new_assembly: String,
    },
    /// Creates a request to change contract ownership
    /// ## Executor
    /// Only the current owner can execute this.
    ProposeNewOwner {
        /// The newly proposed owner
        owner: String,
        /// The validity period of the proposal to change the contract owner
        expires_in: u64,
    },
    /// Removes a request to change contract ownership
    /// ## Executor
    /// Only the current owner can execute this
    DropOwnershipProposal {},
    /// Claims contract ownership
    /// ## Executor
    /// Only the newly proposed owner can execute this
    ClaimOwnership {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ProposalStatus)]
    ProposalState { id: u64 },
}

#[cw_serde]
pub struct MigrateMsg {}

pub use astroport_governance;
