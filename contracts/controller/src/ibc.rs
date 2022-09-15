use cosmwasm_std::{
    entry_point, from_binary, wasm_execute, Addr, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, StdError, StdResult, SubMsg,
};

use astro_satellite_package::IbcAckResult;
use ibc_controller_package::astroport_governance::assembly::ProposalStatus;
use ibc_controller_package::IbcProposal;

use crate::state::{CONFIG, PROPOSAL_STATE};

pub const IBC_APP_VERSION: &str = "astroport-ibc-v1";
pub const IBC_ORDERING: IbcOrder = IbcOrder::Unordered;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> StdResult<IbcChannelOpenResponse> {
    let channel = msg.channel();

    if channel.order != IBC_ORDERING {
        return Err(StdError::generic_err(
            "Ordering is invalid. The channel must be unordered",
        ));
    }
    if channel.version != IBC_APP_VERSION {
        return Err(StdError::generic_err(format!(
            "Must set version to `{}`",
            IBC_APP_VERSION
        )));
    }

    if let Some(counter_version) = msg.counterparty_version() {
        if counter_version != IBC_APP_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{}`",
                IBC_APP_VERSION
            )));
        }
    }

    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_connect(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", &channel.endpoint.channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_receive(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketReceiveMsg,
) -> StdResult<IbcReceiveResponse> {
    unimplemented!()
}

fn confirm_assembly(
    assembly: &Addr,
    proposal_id: u64,
    status: ProposalStatus,
) -> StdResult<SubMsg> {
    Ok(SubMsg::new(wasm_execute(
        assembly,
        &ibc_controller_package::astroport_governance::assembly::ExecuteMsg::IBCProposalCompleted {
            proposal_id,
            status,
        },
        vec![],
    )?))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    let ibc_proposal: IbcProposal = from_binary(&msg.packet.data)?;
    let new_status =
        PROPOSAL_STATE.update(deps.storage, ibc_proposal.id.into(), |state| match state {
            None => Err(StdError::generic_err(format!(
                "Proposal {} was not executed via controller",
                ibc_proposal.id
            ))),
            Some(state) => {
                if state == (ProposalStatus::InProgress {}) {
                    Ok(ProposalStatus::Failed {})
                } else {
                    Err(StdError::generic_err(format!(
                        "Proposal id: {} state is already {}",
                        ibc_proposal.id, state
                    )))
                }
            }
        })?;
    let config = CONFIG.load(deps.storage)?;

    Ok(IbcBasicResponse::new()
        .add_submessage(confirm_assembly(
            &config.assembly,
            ibc_proposal.id,
            new_status,
        )?)
        .add_attribute("action", "packet_timeout")
        .add_attribute("proposal_id", ibc_proposal.id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let ibc_ack: IbcAckResult = from_binary(&msg.acknowledgement.data)?;
    let ibc_proposal: IbcProposal = from_binary(&msg.original_packet.data)?;
    let new_status =
        PROPOSAL_STATE.update(deps.storage, ibc_proposal.id.into(), |state| match state {
            None => Err(StdError::generic_err(format!(
                "Proposal {} was not executed via controller",
                ibc_proposal.id
            ))),
            Some(state) => {
                if state == (ProposalStatus::InProgress {}) {
                    match ibc_ack {
                        IbcAckResult::Ok(_) => Ok(ProposalStatus::Executed {}),
                        IbcAckResult::Error(_) => Ok(ProposalStatus::Failed {}),
                    }
                } else {
                    Err(StdError::generic_err(format!(
                        "Proposal id: {} state is already {}",
                        ibc_proposal.id, state
                    )))
                }
            }
        })?;
    let config = CONFIG.load(deps.storage)?;

    Ok(IbcBasicResponse::new()
        .add_submessage(confirm_assembly(
            &config.assembly,
            ibc_proposal.id,
            new_status,
        )?)
        .add_attribute("action", "packet_ack")
        .add_attribute("proposal_id", ibc_proposal.id.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Err(StdError::generic_err("Closing channel is not allowed"))
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{
        mock_ibc_channel_close_init, mock_ibc_packet_ack, mock_ibc_packet_timeout,
    };
    use cosmwasm_std::{attr, to_binary, Binary, CosmosMsg, IbcAcknowledgement, WasmMsg};

    use ibc_controller_package::ExecuteMsg;

    use crate::contract::execute;
    use crate::test_utils::{init_contract, mock_all, OWNER};

    use super::*;

    fn mock_ibc_execute_proposal(channel_id: &str, proposal_id: u64) -> ExecuteMsg {
        ExecuteMsg::IbcExecuteProposal {
            channel_id: channel_id.to_string(),
            proposal_id,
            messages: vec![],
        }
    }

    #[test]
    fn channel_ack() {
        let (mut deps, env, info) = mock_all(OWNER);
        init_contract(&mut deps, env.clone(), info.clone());

        let channel_id = "channel-0";
        let mut proposal_id = 1;

        let msg = mock_ibc_execute_proposal(channel_id, proposal_id);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Ok acknowledgment
        let ack_msg = mock_ibc_packet_ack(
            channel_id,
            &IbcProposal {
                id: proposal_id,
                messages: vec![],
            },
            IbcAcknowledgement::encode_json(&IbcAckResult::Ok(Binary::default())).unwrap(),
        )
        .unwrap();
        let resp = ibc_packet_ack(deps.as_mut(), env.clone(), ack_msg).unwrap();

        assert!(resp
            .attributes
            .contains(&attr("proposal_id", proposal_id.to_string())));
        let state = PROPOSAL_STATE
            .load(deps.as_ref().storage, proposal_id.into())
            .unwrap();
        assert_eq!(state, ProposalStatus::Executed {});

        assert_eq!(resp.messages.len(), 1);
        let valid_msg = to_binary(
            &ibc_controller_package::astroport_governance::assembly::ExecuteMsg::IBCProposalCompleted {
                proposal_id,
                status: ProposalStatus::Executed,
            },
        )
        .unwrap();
        assert!(matches!(
            &resp.messages[0],
            SubMsg {
                msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }),
                ..
            } if contract_addr == OWNER && msg == &valid_msg
        ));

        // Failed proposal
        proposal_id += 1;
        let msg = mock_ibc_execute_proposal(channel_id, proposal_id);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();
        let ack_msg = mock_ibc_packet_ack(
            channel_id,
            &IbcProposal {
                id: proposal_id,
                messages: vec![],
            },
            IbcAcknowledgement::encode_json(&IbcAckResult::Error("Some error".to_string()))
                .unwrap(),
        )
        .unwrap();
        let resp = ibc_packet_ack(deps.as_mut(), env.clone(), ack_msg).unwrap();

        assert!(resp
            .attributes
            .contains(&attr("proposal_id", proposal_id.to_string())));
        let state = PROPOSAL_STATE
            .load(deps.as_ref().storage, proposal_id.into())
            .unwrap();
        assert_eq!(state, ProposalStatus::Failed {});

        assert_eq!(resp.messages.len(), 1);
        let valid_msg = to_binary(
            &ibc_controller_package::astroport_governance::assembly::ExecuteMsg::IBCProposalCompleted {
                proposal_id,
                status: ProposalStatus::Failed,
            },
        )
        .unwrap();
        assert!(matches!(
            &resp.messages[0],
            SubMsg {
                msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }),
                ..
            } if contract_addr == OWNER && msg == &valid_msg
        ));

        // Previous proposal state was not changed
        let state = PROPOSAL_STATE
            .load(deps.as_ref().storage, (proposal_id - 1).into())
            .unwrap();
        assert_eq!(state, ProposalStatus::Executed {});

        // Proposal with unknown id
        let ack_msg = mock_ibc_packet_ack(
            channel_id,
            &IbcProposal {
                id: 128,
                messages: vec![],
            },
            IbcAcknowledgement::encode_json(&IbcAckResult::Error("Some error".to_string()))
                .unwrap(),
        )
        .unwrap();
        let err = ibc_packet_ack(deps.as_mut(), env, ack_msg).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Proposal 128 was not executed via controller")
        )
    }

    #[test]
    fn channel_timeout() {
        let (mut deps, env, info) = mock_all(OWNER);
        init_contract(&mut deps, env.clone(), info.clone());

        let channel_id = "channel-0";
        let proposal_id = 1;

        let msg = mock_ibc_execute_proposal(channel_id, proposal_id);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let timeout_msg = mock_ibc_packet_timeout(
            &channel_id,
            &IbcProposal {
                id: proposal_id,
                messages: vec![],
            },
        )
        .unwrap();
        let resp = ibc_packet_timeout(deps.as_mut(), env.clone(), timeout_msg.clone()).unwrap();
        assert!(resp
            .attributes
            .contains(&attr("proposal_id", proposal_id.to_string())));

        let state = PROPOSAL_STATE
            .load(deps.as_ref().storage, proposal_id.into())
            .unwrap();
        assert_eq!(state, ProposalStatus::Failed {});

        assert_eq!(resp.messages.len(), 1);
        let valid_msg = to_binary(
            &ibc_controller_package::astroport_governance::assembly::ExecuteMsg::IBCProposalCompleted {
                proposal_id,
                status: ProposalStatus::Failed {},
            },
        )
        .unwrap();
        assert!(matches!(
            &resp.messages[0],
            SubMsg {
                msg: CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, msg, .. }),
                ..
            } if contract_addr == OWNER && msg == &valid_msg
        ));

        // Another timeout packet will fail
        let err = ibc_packet_timeout(deps.as_mut(), env.clone(), timeout_msg).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err(format!(
                "Proposal id: {} state is already {}",
                proposal_id,
                ProposalStatus::Failed {}
            ))
        );

        // timeout msg with unknown proposal id will fail
        let timeout_msg = mock_ibc_packet_timeout(
            &channel_id,
            &IbcProposal {
                id: 128,
                messages: vec![],
            },
        )
        .unwrap();
        let err = ibc_packet_timeout(deps.as_mut(), env.clone(), timeout_msg).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Proposal 128 was not executed via controller")
        )
    }

    #[test]
    fn channel_close() {
        let close_msg =
            mock_ibc_channel_close_init("channel-0", IbcOrder::Unordered, IBC_APP_VERSION);
        let (mut deps, env, _) = mock_all("random");
        let err = ibc_channel_close(deps.as_mut(), env, close_msg).unwrap_err();
        assert_eq!(err, StdError::generic_err("Closing channel is not allowed"))
    }
}
