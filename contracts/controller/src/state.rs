use astro_ibc::astroport_governance::astroport::common::OwnershipProposal;
use astro_ibc::astroport_governance::U64Key;
use astro_ibc::controller::IbcProposalState;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Address that's allowed to change contract parameters
    pub owner: Addr,
    /// when packet times out, measured on remote chain
    pub timeout: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const PROPOSAL_STATE: Map<U64Key, IbcProposalState> = Map::new("proposal_state");

/// Contains a proposal to change contract ownership.
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");
