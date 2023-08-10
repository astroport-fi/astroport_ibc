use astro_satellite_package::MigrateMsg;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, Env};
use cw_storage_plus::Item;

use crate::{
    error::ContractError,
    state::{Config, CONFIG, LATEST_HUB_SIGNAL_TIME},
};
#[cw_serde]
pub struct ConfigV020 {
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

pub const CONFIGV020: Item<ConfigV020> = Item::new("config");

pub fn migrate_to_v100(deps: DepsMut, env: &Env, msg: &MigrateMsg) -> Result<(), ContractError> {
    let old_config = CONFIGV020.load(deps.storage)?;

    let config = Config {
        timeout: old_config.timeout,
        owner: old_config.owner,
        main_maker: old_config.main_maker,
        astro_denom: old_config.astro_denom,
        gov_channel: old_config.gov_channel,
        transfer_channel: old_config.transfer_channel,
        main_controller_port: old_config.main_controller_port,
        max_signal_outage: msg.max_signal_outage,
        emergency_owner: deps.api.addr_validate(&msg.emergency_owner)?,
    };

    CONFIG.save(deps.storage, &config)?;

    LATEST_HUB_SIGNAL_TIME.save(deps.storage, &env.block.time)?;

    Ok(())
}
