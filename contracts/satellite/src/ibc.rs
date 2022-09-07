use std::fmt::Display;

use cosmwasm_std::{
    entry_point, from_binary, to_binary, Binary, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, ReplyOn, StdError, StdResult, SubMsg,
};

use astro_ibc::controller::IbcProposal;
use astro_ibc::satellite::IbcAckResult;
use itertools::Itertools;

use crate::contract::RECEIVE_ID;
use crate::error::{ContractError, Never};
use crate::state::{Config, CONFIG, REPLY_DATA};

pub const IBC_APP_VERSION: &str = "astroport-ibc-v1";
pub const IBC_ORDERING: IbcOrder = IbcOrder::Unordered;

/// Create a serialized success message
pub fn ack_ok() -> Binary {
    to_binary(&IbcAckResult::Ok(b"ok".into())).unwrap()
}

/// Create a serialized error message
pub fn ack_fail(err: impl Display) -> Binary {
    to_binary(&IbcAckResult::Error(err.to_string())).unwrap()
}

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
    deps: DepsMut,
    _env: Env,
    msg: IbcChannelConnectMsg,
) -> Result<IbcBasicResponse, ContractError> {
    let channel = msg.channel();

    let mut config = CONFIG.load(deps.storage)?;
    match config.gov_channel {
        Some(channel_id) => {
            return Err(ContractError::ChannelAlreadyEstablished { channel_id });
        }
        None => {
            if channel.counterparty_endpoint.port_id != config.main_controller_port {
                return Err(ContractError::InvalidSourcePort {
                    invalid: channel.endpoint.port_id.clone(),
                    valid: config.main_controller_port,
                });
            }
        }
    }

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", &channel.endpoint.channel_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
/// We should not return an error if possible, but rather an acknowledgement of failure
pub fn ibc_packet_receive(
    deps: DepsMut,
    _env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    match config.gov_channel {
        Some(gov_channel) if gov_channel != msg.packet.dest.channel_id => {
            return Err(ContractError::InvalidGovernanceChannel {
                invalid: msg.packet.dest.channel_id,
                valid: gov_channel,
            })
        }
        None => return Err(ContractError::GovernanceChannelNotFound {}),
        _ => {}
    }

    let response = IbcReceiveResponse::new().add_attribute("action", "ibc_packet_receive");

    (|| {
        let IbcProposal { id, messages } = from_binary(&msg.packet.data)?;
        let mut response = response.clone().set_ack(ack_ok());
        if !messages.is_empty() {
            let mut messages: Vec<_> = messages
                .into_iter()
                .sorted_by(|a, b| a.order.cmp(&b.order))
                .map(|message| SubMsg::new(message.msg))
                .collect();
            if let Some(last_msg) = messages.last_mut() {
                last_msg.reply_on = ReplyOn::Success;
                last_msg.id = RECEIVE_ID;
            }
            REPLY_DATA.save(deps.storage, &id)?;
            response = response.add_submessages(messages)
        }
        Ok(response)
    })()
    .or_else(|err: ContractError| Ok(response.set_ack(ack_fail(err))))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> Result<IbcBasicResponse, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> Result<IbcBasicResponse, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn ibc_channel_close(
    deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> Result<IbcBasicResponse, ContractError> {
    CONFIG.update::<_, StdError>(deps.storage, |config| {
        config
            .gov_channel
            .ok_or_else(|| StdError::generic_err("Channel was not found"))?;
        Ok(Config {
            gov_channel: None,
            ..config
        })
    })?;

    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_close"))
}
