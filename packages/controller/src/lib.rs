use astroport_governance::assembly::ProposalStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub timeout: u64,
}

#[cw_serde]
pub struct IbcProposal {
    pub id: u64,
    pub messages: Vec<CosmosMsg>,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Executes the IBC proposal that came from Assembly contract
    IbcExecuteProposal {
        channel_id: String,
        proposal_id: u64,
        messages: Vec<CosmosMsg>,
    },
    /// Updates the timeout for the IBC channel
    UpdateTimeout { new_timeout: u64 },
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
    /// Sends heartbeats to specified satellites
    SendHeartbeat { channels: Vec<String> },
}

#[cw_serde]
pub struct MigrateMsg {
    pub new_timeout: Option<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ProposalStatus)]
    ProposalState { id: u64 },

    #[returns(String)]
    LastError {},
}

pub use astroport_governance;
use cosmwasm_std::CosmosMsg;
