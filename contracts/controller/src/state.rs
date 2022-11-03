use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use ap_ibc_controller::astroport_governance::assembly::ProposalStatus;
use ap_ibc_controller::astroport_governance::astroport::common::OwnershipProposal;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    /// Address that's allowed to change contract parameters
    pub owner: Addr,
    /// Assembly address
    pub assembly: Addr,
    /// when packet times out, measured on remote chain
    pub timeout: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const PROPOSAL_STATE: Map<u64, ProposalStatus> = Map::new("proposal_state");

pub const LAST_ERROR: Item<String> = Item::new("last_error");

/// Contains a proposal to change contract ownership.
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");
