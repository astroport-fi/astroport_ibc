use cosmwasm_std::{Addr, DepsMut, Env, StdResult};
use cw_storage_plus::{Item, Map};

use crate::error::ContractError;
use astro_satellite_package::astroport_governance::astroport::common::OwnershipProposal;
use astro_satellite_package::UpdateConfigMsg;
use astroport_ibc::TIMEOUT_LIMITS;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    /// Address which is able to update contracts' parameters
    pub owner: Addr,
    /// ASTRO denom on the remote chain.
    pub astro_denom: String,
    /// Controller contract hosted on the main chain.
    pub main_controller_port: String,
    /// Maker address on the main chain
    pub main_maker: String,
    /// Channel used to interact with assembly contract on the main chain.
    pub gov_channel: Option<String>,
    /// Channel used to transfer Astro tokens
    pub transfer_channel: String,
    /// when packet times out, measured on remote chain
    pub timeout: u64,
}

impl Config {
    pub(crate) fn update(&mut self, params: UpdateConfigMsg) -> Result<(), ContractError> {
        if let Some(astro_denom) = params.astro_denom {
            self.astro_denom = astro_denom;
        }

        if params.gov_channel.is_some()
            && params.accept_new_connections.is_some()
            && params.accept_new_connections.unwrap()
        {
            return Err(ContractError::UpdateChannelError {});
        }

        if let Some(gov_channel) = params.gov_channel {
            self.gov_channel = Some(gov_channel);
        }

        if let Some(accept_new_connections) = params.accept_new_connections {
            if accept_new_connections {
                self.gov_channel = None;
            }
        }

        if let Some(main_controller_addr) = params.main_controller_addr {
            self.main_controller_port = format!("wasm.{main_controller_addr}");
        }

        if let Some(main_maker) = params.main_maker {
            self.main_maker = main_maker;
        }

        if let Some(transfer_channel) = params.transfer_channel {
            self.transfer_channel = transfer_channel;
        }

        if let Some(timeout) = params.timeout {
            if !TIMEOUT_LIMITS.contains(&timeout) {
                return Err(ContractError::TimeoutLimitsError {});
            }
            self.timeout = timeout;
        }

        Ok(())
    }
}

pub const CONFIG: Item<Config> = Item::new("config");

/// Stores map proposal id -> transaction height for successful proposals.
/// Can be considered as a flag to check that proposal was executed.
pub const RESULTS: Map<u64, u64> = Map::new("results");

/// Stores data for reply endpoint.
pub const REPLY_DATA: Item<u64> = Item::new("reply_data");

/// Contains a proposal to change contract ownership.
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");

/// Stores proposal info
pub fn store_proposal(deps: DepsMut, env: Env, proposal_id: u64) -> StdResult<()> {
    RESULTS.save(deps.storage, proposal_id, &env.block.height)
}
