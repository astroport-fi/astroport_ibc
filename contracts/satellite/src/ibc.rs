use std::fmt::Display;

use cosmwasm_std::{
    entry_point, from_json, to_json_binary, Binary, DepsMut, Env, Ibc3ChannelOpenResponse,
    IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg, IbcChannelOpenMsg,
    IbcChannelOpenResponse, IbcOrder, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, ReplyOn, StdError, StdResult, SubMsg,
};

use astro_satellite_package::{IbcAckResult, SatelliteMsg};
use ibc_controller_package::IbcProposal;

use crate::contract::RECEIVE_ID;
use crate::error::{ContractError, Never};
use crate::state::{store_proposal, CONFIG, LATEST_HUB_SIGNAL_TIME, REPLY_DATA};

pub const IBC_APP_VERSION: &str = "astroport-ibc-v1";
pub const IBC_ORDERING: IbcOrder = IbcOrder::Unordered;

/// Create a serialized success message
pub fn ack_ok() -> Binary {
    to_json_binary(&IbcAckResult::Ok(b"ok".into())).unwrap()
}

/// Create a serialized error message
pub fn ack_fail(err: impl Display) -> Binary {
    to_json_binary(&IbcAckResult::Error(err.to_string())).unwrap()
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
            "Must set version to `{IBC_APP_VERSION}`"
        )));
    }

    if let Some(counter_version) = msg.counterparty_version() {
        if counter_version != IBC_APP_VERSION {
            return Err(StdError::generic_err(format!(
                "Counterparty version must be `{IBC_APP_VERSION}`"
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

    if let Some(counter_version) = msg.counterparty_version() {
        if counter_version != IBC_APP_VERSION {
            return Err(ContractError::Std(StdError::generic_err(format!(
                "Counterparty version must be `{IBC_APP_VERSION}`"
            ))));
        }
    }

    let config = CONFIG.load(deps.storage)?;
    match config.gov_channel {
        Some(channel_id) => {
            return Err(ContractError::ChannelAlreadyEstablished { channel_id });
        }
        None => {
            if channel.counterparty_endpoint.port_id != config.main_controller_port {
                return Err(ContractError::InvalidSourcePort {
                    invalid: channel.counterparty_endpoint.port_id.clone(),
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
    env: Env,
    msg: IbcPacketReceiveMsg,
) -> Result<IbcReceiveResponse, Never> {
    do_packet_receive(deps, env, msg).or_else(|err| {
        Ok(IbcReceiveResponse::new()
            .add_attribute("action", "ibc_packet_receive")
            .set_ack(ack_fail(err)))
    })
}

fn do_packet_receive(
    mut deps: DepsMut,
    env: Env,
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

    // TODO: Remove or_else and map_err method calls once the controller is upgraded to v1.0.0
    let satellite_msg = from_json(&msg.packet.data)
        .or_else(|_| {
            let IbcProposal { id, messages } = from_json(&msg.packet.data)?;
            Ok(SatelliteMsg::ExecuteProposal { id, messages })
        })
        .map_err(ContractError::Std)?;

    let mut response = IbcReceiveResponse::new()
        .add_attribute("action", "ibc_packet_receive")
        .set_ack(ack_ok());

    match satellite_msg {
        SatelliteMsg::ExecuteProposal { id, messages } => {
            if !messages.is_empty() {
                let mut messages: Vec<_> = messages.into_iter().map(SubMsg::new).collect();
                if let Some(last_msg) = messages.last_mut() {
                    last_msg.reply_on = ReplyOn::Success;
                    last_msg.id = RECEIVE_ID;
                }
                REPLY_DATA.save(deps.storage, &id)?;
                response = response.add_submessages(messages)
            } else {
                store_proposal(deps.branch(), env.clone(), id)?;
            }
        }
        SatelliteMsg::Heartbeat {} => {}
    }
    LATEST_HUB_SIGNAL_TIME.save(deps.storage, &env.block.time)?;

    Ok(response)
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
    _deps: DepsMut,
    _env: Env,
    _channel: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    Err(StdError::generic_err("Closing channel is not allowed"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{execute, instantiate};
    use astro_satellite_package::{ExecuteMsg, InstantiateMsg, UpdateConfigMsg};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_ibc_channel, mock_ibc_packet_recv, mock_info, MockApi,
        MockQuerier, MockStorage,
    };
    use cosmwasm_std::{CosmosMsg, Empty, MessageInfo, OwnedDeps};

    pub const OWNER: &str = "owner";
    pub const CONTROLLER: &str = "controller";
    pub const GOV_CHANNEL: &str = "channel-20";

    pub fn mock_all(
        sender: &str,
    ) -> (
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
        Env,
        MessageInfo,
    ) {
        let deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(sender, &[]);
        (deps, env, info)
    }

    pub fn init_contract(mut deps: DepsMut, env: Env, info: MessageInfo) {
        let err = instantiate(
            deps.branch(),
            env.clone(),
            info.clone(),
            InstantiateMsg {
                owner: OWNER.to_string(),
                astro_denom: "".to_string(),
                transfer_channel: "".to_string(),
                main_controller: CONTROLLER.to_string(),
                main_maker: "".to_string(),
                timeout: 0,
                max_signal_outage: 1209600,
                emergency_owner: OWNER.to_string(),
            },
        )
        .unwrap_err();
        assert_eq!(ContractError::TimeoutLimitsError {}, err);

        instantiate(
            deps,
            env,
            info,
            InstantiateMsg {
                owner: OWNER.to_string(),
                astro_denom: "".to_string(),
                transfer_channel: "".to_string(),
                main_controller: CONTROLLER.to_string(),
                main_maker: "".to_string(),
                timeout: 60,
                max_signal_outage: 1209600,
                emergency_owner: OWNER.to_string(),
            },
        )
        .unwrap();
    }

    fn mock_ibc_channel_connect_ack(
        my_channel_id: &str,
        order: IbcOrder,
        version: &str,
        their_port: &str,
    ) -> IbcChannelConnectMsg {
        let mut mocked_channel = mock_ibc_channel(my_channel_id, order, version);
        mocked_channel.counterparty_endpoint.port_id = their_port.to_string();
        IbcChannelConnectMsg::new_ack(mocked_channel, version)
    }

    #[test]
    fn channel_open() {
        let (mut deps, env, info) = mock_all(OWNER);
        init_contract(deps.as_mut(), env.clone(), info.clone());

        // Trying to establish channel with wrong remote controller address
        let connect_msg = mock_ibc_channel_connect_ack(
            "channel-0",
            IBC_ORDERING,
            IBC_APP_VERSION,
            "wasm.wrong_controller_addr",
        );
        let err = ibc_channel_connect(deps.as_mut(), env.clone(), connect_msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::InvalidSourcePort {
                invalid: "wasm.wrong_controller_addr".to_string(),
                valid: format!("wasm.{}", CONTROLLER),
            }
        );

        // Correct parameters
        let connect_msg = mock_ibc_channel_connect_ack(
            GOV_CHANNEL,
            IBC_ORDERING,
            IBC_APP_VERSION,
            &format!("wasm.{}", CONTROLLER),
        );
        ibc_channel_connect(deps.as_mut(), env.clone(), connect_msg).unwrap();

        // Setup governance channel
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateConfig(UpdateConfigMsg {
                astro_denom: None,
                gov_channel: Some(GOV_CHANNEL.to_string()),
                main_controller_addr: None,
                main_maker: None,
                transfer_channel: None,
                timeout: None,
                accept_new_connections: None,
                max_signal_outage: None,
                emergency_owner: None,
            }),
        )
        .unwrap();

        // Once gov channel was set up new channels can not be established
        let connect_msg = mock_ibc_channel_connect_ack(
            "channel-21",
            IBC_ORDERING,
            IBC_APP_VERSION,
            &format!("wasm.{}", CONTROLLER),
        );
        let err = ibc_channel_connect(deps.as_mut(), env.clone(), connect_msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::ChannelAlreadyEstablished {
                channel_id: GOV_CHANNEL.to_string()
            }
        )
    }

    #[test]
    fn packet_receive() {
        let (mut deps, env, info) = mock_all(OWNER);
        init_contract(deps.as_mut(), env.clone(), info.clone());

        // Governance channel was not set yet
        let msg = mock_ibc_packet_recv("random_channel", &()).unwrap();
        // However, we will never receive error here, but encoded error message in .acknowledgement field
        let resp = ibc_packet_receive(deps.as_mut(), env.clone(), msg).unwrap();
        let ack: IbcAckResult = from_json(resp.acknowledgement).unwrap();
        assert_eq!(
            ack,
            IbcAckResult::Error("Governance is not established yet".to_string())
        );

        // Setup governance channel
        execute(
            deps.as_mut(),
            env.clone(),
            info,
            ExecuteMsg::UpdateConfig(UpdateConfigMsg {
                astro_denom: None,
                gov_channel: Some(GOV_CHANNEL.to_string()),
                main_controller_addr: None,
                main_maker: None,
                transfer_channel: None,
                timeout: None,
                accept_new_connections: None,
                max_signal_outage: None,
                emergency_owner: None,
            }),
        )
        .unwrap();

        // Trying to send messages via wrong channel
        let msg = mock_ibc_packet_recv("channel-5", &()).unwrap();
        let resp = ibc_packet_receive(deps.as_mut(), env.clone(), msg).unwrap();
        let ack: IbcAckResult = from_json(resp.acknowledgement).unwrap();
        assert_eq!(
            ack,
            IbcAckResult::Error(format!(
                "Invalid governance channel: channel-5. Should be {}",
                GOV_CHANNEL
            ))
        );

        let ibc_proposal = IbcProposal {
            id: 1,
            messages: vec![
                // pass any valid CosmosMsg message.
                // The meaning of this msg doesn't matter as this is just a unit test
                CosmosMsg::Custom(Empty {}),
            ],
        };
        // Send messages via governance channel
        let msg = mock_ibc_packet_recv(GOV_CHANNEL, &ibc_proposal).unwrap();
        let resp = ibc_packet_receive(deps.as_mut(), env.clone(), msg).unwrap();
        assert_eq!(resp.messages.last().unwrap().reply_on, ReplyOn::Success);
        let ack: IbcAckResult = from_json(&resp.acknowledgement).unwrap();
        assert_eq!(ack, IbcAckResult::Ok(b"ok".into()));
    }
}
