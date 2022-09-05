use astro_ibc::astroport_governance::U64Key;
use cosmwasm_std::{Addr, Env, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Controller contract hosted on the main chain.
    pub owner: Addr,
    /// when packet times out, measured on remote chain
    pub timeout: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const CHANNEL: Item<String> = Item::new("channel");
