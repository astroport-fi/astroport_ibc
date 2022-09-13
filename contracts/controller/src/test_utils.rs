use crate::contract::instantiate;
use astro_ibc::controller::InstantiateMsg;
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{Env, MessageInfo, OwnedDeps};

pub const OWNER: &str = "owner";

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

pub fn init_contract(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    env: Env,
    info: MessageInfo,
) {
    instantiate(
        deps.as_mut(),
        env,
        info,
        InstantiateMsg {
            owner: OWNER.to_string(),
            assembly: OWNER.to_string(),
            timeout: 360,
        },
    )
    .unwrap();
}
