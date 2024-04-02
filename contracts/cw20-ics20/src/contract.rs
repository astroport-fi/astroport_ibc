use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::{get_contract_version, set_contract_version};
use cw20_ics20::msg::{ExecuteMsg, InitMsg, QueryMsg};

const CONTRACT_NAME: &str = "crates.io:cw20-ics20";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<Response> {
    unimplemented!(
        "This is transition version of Astroport's cw20_ics20 which cannot be instantiated"
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> StdResult<Response> {
    Err(StdError::generic_err(
        "This is transition version of Astroport's cw20_ics20. All execute endpoints are disabled. Currently it only receives IBCed ASTRO from remote chains",
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    cw20_ics20::contract::query(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> StdResult<Response> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "crates.io:cw20-ics20" => match contract_version.version.as_ref() {
            // 0.13.4 - testnet
            // 0.15.1 - mainnet
            "0.13.4" | "0.15.1" => {}
            _ => {
                return Err(StdError::generic_err(format!(
                    "Unsupported version: {}",
                    contract_version.version
                )))
            }
        },
        _ => {
            return Err(StdError::generic_err(format!(
                "Unsupported contract {}",
                contract_version.contract
            )))
        }
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
