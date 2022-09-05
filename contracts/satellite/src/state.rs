use astro_ibc::astroport_governance::U64Key;
use cosmwasm_std::{Env, Uint64};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Controller contract hosted on the main chain.
    pub main_controller_port: String,
    /// Current channel used to interact with the main chain.
    pub channel: Option<String>,
}

/// Structure to point to exact transaction in history.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TxInfo {
    pub height: u64,
    pub tx_index: u32,
}

impl From<Env> for TxInfo {
    fn from(env: Env) -> Self {
        Self {
            height: env.block.height,
            tx_index: env.transaction.unwrap().index,
        }
    }
}

pub const CONFIG: Item<Config> = Item::new("config");

/// Stores map SequenceId -> Transaction info for successful proposals.
/// Can be considered as a flag to check that proposal was executed.
pub const RESULTS: Map<U64Key, TxInfo> = Map::new("results");

/// Stores data for reply endpoint.
pub const REPLY_DATA: Item<u64> = Item::new("reply_data");
