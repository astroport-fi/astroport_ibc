use crate::state::{Config, CONFIG};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult};
use cw_storage_plus::Item;

pub fn migrate_config(deps: &mut DepsMut) -> StdResult<()> {
    #[cw_serde]
    struct ConfigV01 {
        pub owner: Addr,
        pub assembly: Addr,
        pub timeout: u64,
    }

    let config: ConfigV01 = Item::new("config").load(deps.storage)?;
    let new_config = Config {
        owner: config.owner,
        timeout: config.timeout,
    };

    CONFIG.save(deps.storage, &new_config)?;

    Ok(())
}
