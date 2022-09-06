use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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
