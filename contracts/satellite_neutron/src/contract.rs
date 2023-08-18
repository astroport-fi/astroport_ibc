#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{coin, DepsMut, Env, MessageInfo, Response, StdError, StdResult};
use cw2::{get_contract_version, set_contract_version};
use cw_utils::must_pay;
use neutron_sdk::bindings::msg::{IbcFee, NeutronMsg};
use neutron_sdk::bindings::query::NeutronQuery;
use neutron_sdk::query::min_ibc_fee::query_min_ibc_fee;
use neutron_sdk::sudo::msg::{RequestPacketTimeoutHeight, TransferSudoMsg};

use astro_satellite::contract::check_messages;
use astro_satellite::error::ContractError;
use astro_satellite::state::{Config, CONFIG, OWNERSHIP_PROPOSAL};
use astro_satellite_package::astroport_governance::astroport::common::{
    claim_ownership, drop_ownership_proposal, propose_new_owner,
};
use astro_satellite_package::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use astroport_ibc::TIMEOUT_LIMITS;

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// This contract accepts only one fee denom
const FEE_DENOM: &str = "untrn";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<NeutronQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    if !TIMEOUT_LIMITS.contains(&msg.timeout) {
        return Err(ContractError::TimeoutLimitsError {});
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
        },
    )?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<NeutronMsg>,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        ExecuteMsg::TransferAstro {} => {
            let config = CONFIG.load(deps.storage)?;
            let amount = must_pay(&info, &config.astro_denom)?;
            let fee = min_ntrn_ibc_fee(
                query_min_ibc_fee(deps.as_ref())
                    .map_err(|err| StdError::generic_err(err.to_string()))?
                    .min_fee,
            );
            let msg = NeutronMsg::IbcTransfer {
                source_port: "transfer".to_string(),
                source_channel: config.transfer_channel,
                sender: env.contract.address.to_string(),
                receiver: config.main_maker,
                token: coin(amount.u128(), &config.astro_denom),
                timeout_height: RequestPacketTimeoutHeight {
                    revision_number: None,
                    revision_height: None,
                },
                // Neutron expects nanoseconds
                // https://github.com/neutron-org/neutron/blob/303d764b57d871749fcf7d59a67b5d3078779258/proto/transfer/v1/tx.proto#L39-L42
                timeout_timestamp: env.block.time.plus_seconds(config.timeout).nanos(),
                memo: "".to_string(),
                fee,
            };
            Ok(Response::new()
                .add_message(msg)
                .add_attribute("action", "transfer_astro"))
        }
        ExecuteMsg::UpdateConfig(params) => {
            CONFIG.update(deps.storage, |mut config| {
                if config.owner == info.sender {
                    config.update(params)?;
                    Ok(config)
                } else {
                    Err(ContractError::Unauthorized {})
                }
            })?;
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
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(_deps: DepsMut, _env: Env, _msg: TransferSudoMsg) -> StdResult<Response> {
    // Neutron requires sudo endpoint to be implemented
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let contract_version = get_contract_version(deps.storage)?;

    match contract_version.contract.as_ref() {
        "astro-satellite" => match contract_version.version.as_ref() {
            "0.2.0" => {}
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

fn min_ntrn_ibc_fee(fee: IbcFee) -> IbcFee {
    IbcFee {
        recv_fee: fee.recv_fee,
        ack_fee: fee
            .ack_fee
            .into_iter()
            .filter(|a| a.denom == FEE_DENOM)
            .collect(),
        timeout_fee: fee
            .timeout_fee
            .into_iter()
            .filter(|a| a.denom == FEE_DENOM)
            .collect(),
    }
}

#[cfg(test)]
mod testing {
    use std::marker::PhantomData;

    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, to_binary, ContractResult, CosmosMsg, OwnedDeps, SystemResult};
    use neutron_sdk::query::min_ibc_fee::MinIbcFeeResponse;

    use super::*;

    fn mock_neutron_dependencies(
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier<NeutronQuery>, NeutronQuery> {
        let neutron_custom_handler = |request: &NeutronQuery| {
            let contract_result: ContractResult<_> = match request {
                NeutronQuery::MinIbcFee {} => to_binary(&MinIbcFeeResponse {
                    min_fee: IbcFee {
                        recv_fee: vec![],
                        ack_fee: coins(10000, FEE_DENOM),
                        timeout_fee: coins(10000, FEE_DENOM),
                    },
                })
                .into(),
                _ => unimplemented!("Unsupported query request: {:?}", request),
            };
            SystemResult::Ok(contract_result)
        };

        OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::new(&[]).with_custom_handler(neutron_custom_handler),
            custom_query_type: PhantomData,
        }
    }

    #[test]
    fn test_transfer_astro() {
        let owner = "owner";
        let astro_denom = "ibc/astro";
        let transfer_channel = "channel-1";
        let main_maker = "wasm1hub_maker";
        let timeout = 60;

        let mut deps = mock_neutron_dependencies();
        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(owner, &[]),
            InstantiateMsg {
                owner: owner.to_string(),
                astro_denom: astro_denom.to_string(),
                transfer_channel: transfer_channel.to_string(),
                main_controller: "".to_string(),
                main_maker: main_maker.to_string(),
                timeout,
            },
        )
        .unwrap();

        let env = mock_env();
        let resp = execute(
            deps.as_mut(),
            env.clone(),
            mock_info(owner, &coins(1000, astro_denom)),
            ExecuteMsg::TransferAstro {},
        )
        .unwrap();

        assert_eq!(resp.messages.len(), 1);
        assert_eq!(
            resp.messages[0].msg,
            CosmosMsg::Custom(NeutronMsg::IbcTransfer {
                source_port: "transfer".to_string(),
                source_channel: transfer_channel.to_string(),
                sender: "cosmos2contract".to_string(),
                receiver: main_maker.to_string(),
                token: coin(1000, astro_denom),
                timeout_height: RequestPacketTimeoutHeight {
                    revision_number: None,
                    revision_height: None,
                },
                timeout_timestamp: env.block.time.plus_seconds(timeout).nanos(),
                memo: "".to_string(),
                fee: IbcFee {
                    recv_fee: vec![],
                    ack_fee: coins(10000, FEE_DENOM),
                    timeout_fee: coins(10000, FEE_DENOM),
                },
            })
        );
    }
}
