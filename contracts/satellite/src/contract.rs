#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, wasm_execute, Addr, Api, Binary, CosmosMsg, CustomMsg, Deps, DepsMut,
    Env, IbcMsg, IbcTimeout, MessageInfo, QuerierWrapper, Reply, Response, StdError, WasmMsg,
};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Map;

use astro_satellite_package::astroport_governance::astroport::common::{
    claim_ownership, drop_ownership_proposal, propose_new_owner,
};
use astro_satellite_package::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::state::{
    instantiate_state, set_emergency_owner_as_admin, store_proposal, update_config, CONFIG,
    OWNERSHIP_PROPOSAL, REPLY_DATA, RESULTS,
};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const RECEIVE_ID: u64 = 1;

#[cfg_attr(all(not(feature = "library")), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    instantiate_state(deps, env, msg)?;

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

#[cfg_attr(all(not(feature = "library")), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TransferAstro {} => {
            let config = CONFIG.load(deps.storage)?;

            // Query and send the whole astro balance
            let astro_balance = deps
                .querier
                .query_balance(&env.contract.address, &config.astro_denom)?;

            if astro_balance.amount.is_zero() {
                return Err(ContractError::NoAstroBalance {});
            }

            let msg = CosmosMsg::Ibc(IbcMsg::Transfer {
                channel_id: config.transfer_channel,
                to_address: config.main_maker,
                amount: astro_balance,
                timeout: IbcTimeout::from(env.block.time.plus_seconds(config.timeout)),
            });
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "transfer_astro"))
        }
        ExecuteMsg::UpdateConfig(params) => update_config(deps, info, env, params),
        ExecuteMsg::CheckMessages(messages) => check_messages(deps.api, env, messages),
        ExecuteMsg::ExecuteFromMultisig(proposal_messages) => {
            exec_from_multisig(deps.querier, info, env, proposal_messages)
        }
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
        ExecuteMsg::SetEmergencyOwnerAsAdmin {} => set_emergency_owner_as_admin(deps, env, info),
    }
}

/// Checks that proposal messages are correct.
pub fn check_messages<M>(
    api: &dyn Api,
    env: Env,
    mut messages: Vec<CosmosMsg<M>>,
) -> Result<Response<M>, ContractError>
where
    M: CustomMsg,
{
    messages.iter().try_for_each(|msg| match msg {
        CosmosMsg::Wasm(
            WasmMsg::Migrate { contract_addr, .. } | WasmMsg::UpdateAdmin { contract_addr, .. },
        ) if api.addr_validate(contract_addr)? == env.contract.address => {
            Err(StdError::generic_err(
                "Can't check messages with a migration or update admin message of the contract itself",
            ))
        }
        CosmosMsg::Stargate { type_url, .. } if type_url.contains("MsgGrant") => Err(
            StdError::generic_err("Can't check messages with a MsgGrant message"),
        ),
        _ => Ok(()),
    })?;

    messages.push(
        wasm_execute(
            env.contract.address,
            &ExecuteMsg::<M>::CheckMessagesPassed {},
            vec![],
        )?
        .into(),
    );

    Ok(Response::new()
        .add_attribute("action", "check_messages")
        .add_messages(messages))
}

pub fn exec_from_multisig<M>(
    querier: QuerierWrapper,
    info: MessageInfo,
    env: Env,
    messages: Vec<CosmosMsg<M>>,
) -> Result<Response<M>, ContractError>
where
    M: CustomMsg,
{
    match querier
        .query_wasm_contract_info(&env.contract.address)?
        .admin
    {
        None => Err(ContractError::Unauthorized {}),
        // Don't allow to execute this endpoint if the contract is admin of itself
        Some(admin) if (admin != info.sender || admin == env.contract.address) => {
            Err(ContractError::Unauthorized {})
        }
        _ => Ok(()),
    }?;

    Ok(Response::new().add_messages(messages))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ProposalState { id } => {
            let state = RESULTS.load(deps.storage, id)?;
            Ok(to_json_binary(&state)?)
        }
    }
}

#[cfg_attr(all(not(feature = "library")), entry_point)]
pub fn migrate(mut deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    let response = match contract_version.contract.as_ref() {
        "astro-satellite" => match contract_version.version.as_ref() {
            "1.1.0-hubmove" => {
                set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
                Response::new()
            }
            _ => return Err(ContractError::MigrationError {}),
        },
        "astro-assembly" => match contract_version.version.as_ref() {
            "1.3.2" => {
                // Converting Assembly on Terra into Satellite.
                // Tested on forked Terra mainnet. Gas used: 2_492_503. State was successfully wiped out.
                // ibc port was obtained.

                ensure!(
                    env.block.chain_id == "pisco-1" || env.block.chain_id == "phoenix-1",
                    StdError::generic_err(
                        "Assembly -> Satellite conversion is only available on Terra"
                    )
                );

                let init_msg = msg
                    .init_msg
                    .ok_or_else(|| StdError::generic_err("init_msg must be set on Terra"))?;

                deps.storage.remove(b"proposal_count");
                let proposals: Map<u64, ()> = Map::new("proposals");
                proposals.clear(deps.storage);

                let cw_admin = deps
                    .querier
                    .query_wasm_contract_info(&env.contract.address)?
                    .admin
                    .unwrap();
                // Even though info object is ignored in instantiate, we provide it for clarity
                let info = MessageInfo {
                    sender: Addr::unchecked(cw_admin),
                    funds: vec![],
                };
                // Instantiate Satellite state.
                // Config and cw2 info will be overwritten.
                instantiate(deps.branch(), env, info, init_msg)?
            }
            _ => return Err(ContractError::MigrationError {}),
        },
        _ => return Err(ContractError::MigrationError {}),
    };

    Ok(response.add_attributes([
        ("previous_contract_name", contract_version.contract.as_str()),
        (
            "previous_contract_version",
            contract_version.version.as_str(),
        ),
        ("new_contract_name", CONTRACT_NAME),
        ("new_contract_version", CONTRACT_VERSION),
    ]))
}
