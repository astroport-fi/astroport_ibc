#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, Coin, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Reply, Response,
};
use cw2::set_contract_version;
use cw_utils::must_pay;

use crate::error::ContractError;
use crate::state::{Config, CONFIG, REPLY_DATA, RESULTS};
use astro_ibc::satellite::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const RECEIVE_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.save(
        deps.storage,
        &Config {
            astro_denom: msg.astro_denom,
            main_controller_port: format!("wasm.{}", msg.main_controller),
            main_maker: msg.main_maker,
            gov_channel: None,
            transfer_channel: msg.transfer_channel,
            timeout: msg.timeout,
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let proposal_id = REPLY_DATA.load(deps.storage)?.into();
    match reply.id {
        RECEIVE_ID => {
            RESULTS.save(deps.storage, proposal_id, &env.into())?;
            Ok(Response::new())
        }
        _ => Err(ContractError::InvalidReplyId {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TransferAstro {} => {
            let config = CONFIG.load(deps.storage)?;
            let amount = must_pay(&info, &config.astro_denom)?;
            let msg = CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id: config.transfer_channel,
                to_address: config.main_maker,
                amount: Coin {
                    denom: config.astro_denom.clone(),
                    amount,
                },
                timeout: IbcTimeout::from(env.block.time.plus_seconds(config.timeout)),
            });
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "transfer_astro"))
        }
        ExecuteMsg::UpdateConfig { update_params } => {
            CONFIG.update(deps.storage, |mut config| {
                if env.contract.address == info.sender {
                    config.update(update_params);
                    Ok(config)
                } else {
                    Err(ContractError::Unauthorized {})
                }
            })?;
            Ok(Response::new().add_attribute("action", "update_config"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> Result<Binary, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
