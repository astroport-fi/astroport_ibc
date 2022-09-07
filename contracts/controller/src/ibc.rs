use cosmwasm_std::{
    entry_point, from_binary, DepsMut, Env, Ibc3ChannelOpenResponse, IbcBasicResponse,
    IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg, IbcChannelOpenResponse, IbcOrder,
    IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg, IbcReceiveResponse, StdError,
    StdResult,
};

use astro_ibc::controller::{IbcProposal, IbcProposalState};
use astro_ibc::satellite::IbcAckResult;

use crate::state::PROPOSAL_STATE;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    let ibc_proposal: IbcProposal = from_binary(&msg.packet.data)?;
    PROPOSAL_STATE.update(deps.storage, ibc_proposal.id.into(), |state| match state {
        None => Err(StdError::generic_err(format!(
            "Proposal {} was not executed via controller",
            ibc_proposal.id
        ))),
        Some(state) => {
            if state == (IbcProposalState::InProgress {}) {
                Ok(IbcProposalState::Failed {})
            } else {
                Err(StdError::generic_err(format!(
                    "Proposal id: {} state is already {}",
                    ibc_proposal.id, state
                )))
            }
        }
    })?;
    Ok(IbcBasicResponse::new().add_attribute("action", "packet_timeout"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    let ibc_ack: IbcAckResult = from_binary(&msg.acknowledgement.data)?;
    let ibc_proposal: IbcProposal = from_binary(&msg.original_packet.data)?;
    PROPOSAL_STATE.update(deps.storage, ibc_proposal.id.into(), |state| match state {
        None => Err(StdError::generic_err(format!(
            "Proposal {} was not executed via controller",
            ibc_proposal.id
        ))),
        Some(state) => {
            if state == (IbcProposalState::InProgress {}) {
                match ibc_ack {
                    IbcAckResult::Ok(_) => Ok(IbcProposalState::Succeed {}),
                    IbcAckResult::Error(_) => Ok(IbcProposalState::Failed {}),
                }
            } else {
                Err(StdError::generic_err(format!(
                    "Proposal id: {} state is already {}",
                    ibc_proposal.id, state
                )))
            }
        }
    })?;
    Ok(IbcBasicResponse::new().add_attribute("action", "packet_ack"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Err(StdError::generic_err("Closing channel is not allowed"))
}
