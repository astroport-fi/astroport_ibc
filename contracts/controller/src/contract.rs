use ap_ibc_controller::astroport_governance::assembly::ProposalStatus;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcTimeout, MessageInfo, Response,
    StdError,
};
use cw2::{get_contract_version, set_contract_version, ContractVersion};

use ap_ibc_controller::astroport_governance::astroport::asset::addr_validate_to_lower;
use ap_ibc_controller::astroport_governance::astroport::common::{
    claim_ownership, drop_ownership_proposal, propose_new_owner,
};
use ap_ibc_controller::{ExecuteMsg, IbcProposal, InstantiateMsg, MigrateMsg, QueryMsg};

use crate::error::ContractError;
use crate::state::{Config, CONFIG, OWNERSHIP_PROPOSAL, PROPOSAL_STATE};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

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
            owner: addr_validate_to_lower(deps.api, &msg.owner)?,
            assembly: addr_validate_to_lower(deps.api, &msg.assembly)?,
            timeout: msg.timeout,
        },
    )?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    match msg {
        ExecuteMsg::IbcExecuteProposal {
            channel_id,
            proposal_id,
            messages,
        } => {
            if config.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            let ibc_msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id: channel_id.clone(),
                data: to_binary(&IbcProposal {
                    id: proposal_id,
                    messages,
                })?,
                timeout: IbcTimeout::from(env.block.time.plus_seconds(config.timeout)),
            });
            PROPOSAL_STATE.save(deps.storage, proposal_id, &ProposalStatus::InProgress {})?;

            Ok(Response::new()
                .add_message(ibc_msg)
                .add_attribute("action", "ibc_execute")
                .add_attribute("channel", channel_id))
        }
        ExecuteMsg::UpdateConfig { new_assembly } => CONFIG
            .update(deps.storage, |mut config| {
                if info.sender == config.owner {
                    config.assembly = addr_validate_to_lower(deps.api, &new_assembly)?;
                    Ok(config)
                } else {
                    Err(ContractError::Unauthorized {})
                }
            })
            .map(|_| Response::new().add_attribute("action", "update_config")),
        ExecuteMsg::ProposeNewOwner { owner, expires_in } => propose_new_owner(
            deps,
            info,
            env,
            owner,
            expires_in,
            config.owner,
            OWNERSHIP_PROPOSAL,
        )
        .map_err(Into::into),
        ExecuteMsg::DropOwnershipProposal {} => {
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ProposalState { id } => {
            let state = PROPOSAL_STATE.load(deps.storage, id)?;
            Ok(to_binary(&state)?)
        }
    }
}

/// Manages contract migration.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let ContractVersion {
        contract: contract_name,
        version,
    } = get_contract_version(deps.storage)?;

    match contract_name.as_ref() {
        "ibc-controller" => match version.as_ref() {
            "0.1.0" => {}
            _ => return Err(ContractError::MigrationError {}),
        },
        "astroport-ibc-controller" => {
            // Future migrations are added here
            return Err(ContractError::MigrationError {});
        }
        _ => return Err(ContractError::MigrationError {}),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new()
        .add_attribute("previous_contract_name", contract_name)
        .add_attribute("previous_contract_version", version)
        .add_attribute("new_contract_name", CONTRACT_NAME)
        .add_attribute("new_contract_version", CONTRACT_VERSION))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, BankMsg, Coin, Uint128, Uint64};

    use crate::test_utils::{init_contract, mock_all, OWNER};
    use ap_ibc_controller::astroport_governance::assembly::ProposalMessage;

    use super::*;

    #[test]
    fn test_ibc_execute() {
        let (mut deps, env, info) = mock_all(OWNER);

        init_contract(&mut deps, env.clone(), info.clone());

        let channel_id = "channel-0".to_string();
        let proposal_id = 1;
        let proposal_msg = ProposalMessage {
            order: Uint64::new(1),
            msg: CosmosMsg::Bank(BankMsg::Send {
                to_address: "foreign_addr".to_string(),
                amount: vec![Coin {
                    denom: "stake".to_string(),
                    amount: Uint128::new(100),
                }],
            }),
        };
        let msg = ExecuteMsg::IbcExecuteProposal {
            channel_id,
            proposal_id,
            messages: vec![proposal_msg.clone()],
        };
        let resp = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();

        assert_eq!(resp.messages.len(), 1);
        let real_timeout = IbcTimeout::with_timestamp(env.block.time.plus_seconds(360));
        match &resp.messages[0].msg {
            CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id,
                timeout,
                data,
            }) if channel_id == channel_id && timeout == &real_timeout => {
                let proposal: IbcProposal = from_binary(&data).unwrap();
                assert_eq!(proposal.id, proposal_id);
                assert_eq!(proposal.messages.len(), 1);
                assert_eq!(proposal.messages[0], proposal_msg);
            }
            _ => panic!("Unexpected message"),
        }

        let state = PROPOSAL_STATE
            .load(deps.as_ref().storage, proposal_id.into())
            .unwrap();
        assert_eq!(state, ProposalStatus::InProgress {})
    }
}
