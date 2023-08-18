#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, wasm_execute, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout,
    MessageInfo, Reply, Response, StdError, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_utils::must_pay;

use astro_satellite_package::astroport_governance::astroport::common::{
    claim_ownership, drop_ownership_proposal, propose_new_owner,
};
use astro_satellite_package::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use astroport_ibc::{SIGNAL_OUTAGE_LIMITS, TIMEOUT_LIMITS};

use crate::error::ContractError;
use crate::migration::migrate_to_v100;
use crate::state::{
    store_proposal, Config, CONFIG, LATEST_HUB_SIGNAL_TIME, OWNERSHIP_PROPOSAL, REPLY_DATA, RESULTS,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) const RECEIVE_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if !TIMEOUT_LIMITS.contains(&msg.timeout) {
        return Err(ContractError::TimeoutLimitsError {});
    }

    if !SIGNAL_OUTAGE_LIMITS.contains(&msg.max_signal_outage) {
        return Err(ContractError::SignalOutageLimitsError {});
    }

    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_validate(&msg.owner)?,
            astro_denom: msg.astro_denom,
            main_controller_port: format!("wasm.{}", msg.main_controller),
            main_maker: msg.main_maker,
            gov_channel: None,
            transfer_channel: msg.transfer_channel,
            timeout: msg.timeout,
            max_signal_outage: msg.max_signal_outage,
            emergency_owner: deps.api.addr_validate(&msg.emergency_owner)?,
        },
    )?;

    LATEST_HUB_SIGNAL_TIME.save(deps.storage, &env.block.time)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    let proposal_id = REPLY_DATA.load(deps.storage)?;
    match reply.id {
        RECEIVE_ID => {
            store_proposal(deps, env, proposal_id)?;
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
        ExecuteMsg::UpdateConfig(params) => {
            let mut config = CONFIG.load(deps.storage)?;
            if !(info.sender == config.owner
                || LATEST_HUB_SIGNAL_TIME
                    .load(deps.storage)?
                    .plus_seconds(config.max_signal_outage)
                    < env.block.time
                    && info.sender == config.emergency_owner)
            {
                return Err(ContractError::Unauthorized {});
            }
            config.update(deps.api, params)?;
            CONFIG.save(deps.storage, &config)?;

            Ok(Response::new().add_attribute("action", "update_config"))
        }
        ExecuteMsg::CheckMessages(proposal_messages) => check_messages(env, proposal_messages),
        ExecuteMsg::CheckMessagesPassed {} => Err(ContractError::MessagesCheckPassed {}),
        ExecuteMsg::ProposeNewOwner { owner, expires_in } => {
            let config = CONFIG.load(deps.storage)?;

            propose_new_owner(
                deps,
                info,
                env,
                owner,
                expires_in,
                config.owner,
                OWNERSHIP_PROPOSAL,
            )
            .map_err(Into::into)
        }
        ExecuteMsg::DropOwnershipProposal {} => {
            let config = CONFIG.load(deps.storage)?;

            drop_ownership_proposal(deps, info, config.owner, OWNERSHIP_PROPOSAL)
                .map_err(Into::into)
        }
        ExecuteMsg::ClaimOwnership {} => {
            claim_ownership(deps, info, env, OWNERSHIP_PROPOSAL, |deps, new_owner| {
                CONFIG
                    .update::<_, StdError>(deps.storage, |mut v| {
                        v.owner = new_owner;
                        Ok(v)
                    })
                    .map(|_| ())
            })
            .map_err(Into::into)
        }
        ExecuteMsg::SetEmergencyOwnerAsAdmin {} => {
            let config = CONFIG.load(deps.storage)?;
            if LATEST_HUB_SIGNAL_TIME
                .load(deps.storage)?
                .plus_seconds(config.max_signal_outage)
                < env.block.time
                && info.sender == config.emergency_owner
            {
                Ok(Response::new().add_message(WasmMsg::UpdateAdmin {
                    contract_addr: env.contract.address.to_string(),
                    admin: config.emergency_owner.to_string(),
                }))
            } else {
                Err(ContractError::Unauthorized {})
            }
        }
    }
}

fn check_messages(env: Env, mut messages: Vec<CosmosMsg>) -> Result<Response, ContractError> {
    messages.push(CosmosMsg::Wasm(wasm_execute(
        env.contract.address,
        &ExecuteMsg::CheckMessagesPassed {},
        vec![],
    )?));

    Ok(Response::new()
        .add_attribute("action", "check_messages")
        .add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ProposalState { id } => {
            let state = RESULTS.load(deps.storage, id)?;
            Ok(to_binary(&state)?)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(mut deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "astro-satellite" => match contract_version.version.as_ref() {
            "0.2.0" => {
                migrate_to_v100(deps.branch(), &env, &msg)?;
            }
            _ => return Err(ContractError::MigrationError {}),
        },
        _ => return Err(ContractError::MigrationError {}),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", &contract_version.contract)
        .add_attribute("previous_contract_version", &contract_version.version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}
