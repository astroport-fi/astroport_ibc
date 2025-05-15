use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

use ibc_controller_package::astroport_governance::assembly::ProposalStatus;
use astroport::common::OwnershipProposal;

#[cw_serde]
pub struct Config {
    /// Address which is able to run IBC proposals
    pub owner: Addr,
    /// when packet times out, measured on remote chain
    pub timeout: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const PROPOSAL_STATE: Map<u64, ProposalStatus> = Map::new("proposal_state");

pub const LAST_ERROR: Item<String> = Item::new("last_error");

/// Contains a proposal to change contract ownership.
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");
