use astroport_governance::assembly::ProposalStatus;
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CosmosMsg, CustomMsg, Empty};

#[cw_serde]
pub struct InstantiateMsg {
    /// Address which is able to update contracts' parameters
    pub owner: String,
    /// ASTRO denom on the remote chain.
    pub astro_denom: String,
    /// Channel used to transfer Astro tokens
    pub transfer_channel: String,
    /// Controller contract hosted on the main chain.
    pub main_controller: String,
    /// Maker address on the main chain
    pub main_maker: String,
    /// when packet times out, measured on remote chain
    pub timeout: u64,
    /// Time in seconds after which the satellite considers itself lost
    pub max_signal_outage: u64,
    /// An address that can migrate the contract and change its config if the satellite is lost
    pub emergency_owner: String,
}

#[cw_serde]
pub struct UpdateConfigMsg {
    pub astro_denom: Option<String>,
    pub gov_channel: Option<String>,
    pub main_controller_addr: Option<String>,
    pub main_maker: Option<String>,
    pub transfer_channel: Option<String>,
    pub accept_new_connections: Option<bool>,
    pub timeout: Option<u64>,
    pub max_signal_outage: Option<u64>,
    pub emergency_owner: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg<M: CustomMsg = Empty> {
    TransferAstro {},
    UpdateConfig(UpdateConfigMsg),
    CheckMessages(Vec<CosmosMsg<M>>),
    ExecuteFromMultisig(Vec<CosmosMsg<M>>),
    CheckMessagesPassed {},
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
    /// It sets the emergency owner as admin of the contract to migrate it if the satellite is lost
    SetEmergencyOwnerAsAdmin {},
}

#[cw_serde]
pub enum SatelliteMsg {
    ExecuteProposal { id: u64, messages: Vec<CosmosMsg> },
    Heartbeat {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ProposalStatus)]
    ProposalState { id: u64 },
}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// This is compatible with the JSON serialization
#[cw_serde]
pub enum IbcAckResult {
    Ok(Binary),
    Error(String),
}

pub use astroport_governance;
