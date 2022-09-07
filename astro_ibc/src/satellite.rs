use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UpdateConfigMsg {
    pub astro_denom: Option<String>,
    pub main_controller_port: Option<String>,
    pub main_maker: Option<String>,
    pub transfer_channel: Option<String>,
    pub timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    TransferAstro {},
    UpdateConfig { update_params: UpdateConfigMsg },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}

/// This is a generic ICS acknowledgement format.
/// Proto defined here: https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
/// This is compatible with the JSON serialization
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub enum IbcAckResult {
    Ok(Binary),
    Error(String),
}
