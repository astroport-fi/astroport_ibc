use astro_satellite::contract::{execute, instantiate, query, reply};
use astro_satellite::error::ContractError;
use astro_satellite::state::Config;
use astro_satellite_package::{ExecuteMsg, InstantiateMsg, UpdateConfigMsg};
use cosmwasm_std::{
    from_slice, wasm_execute, Addr, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult,
};

use cw_multi_test::{App, Contract, ContractWrapper, Executor};

fn mock_app(owner: &Addr, coins: Vec<Coin>) -> App {
    App::new(|router, _, storage| {
        // initialization moved to App construction
        router.bank.init_balance(storage, owner, coins).unwrap()
    })
}

fn satellite_contract() -> Box<dyn Contract<Empty>> {
    Box::new(ContractWrapper::new_with_empty(execute, instantiate, query).with_reply_empty(reply))
}

fn noop_contract() -> Box<dyn Contract<Empty>> {
    fn noop_execute(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        _msg: Empty,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn noop_query(_deps: Deps, _env: Env, _msg: Empty) -> StdResult<Binary> {
        Ok(Default::default())
    }

    Box::new(ContractWrapper::new_with_empty(
        noop_execute,
        noop_execute,
        noop_query,
    ))
}

#[test]
fn test_check_messages() {
    let owner = Addr::unchecked("owner");
    let mut app = mock_app(&owner, vec![]);

    let satellite_code = app.store_code(satellite_contract());
    let err = app
        .instantiate_contract(
            satellite_code,
            owner.clone(),
            &InstantiateMsg {
                owner: owner.to_string(),
                astro_denom: "none".to_string(),
                transfer_channel: "none".to_string(),
                main_controller: "none".to_string(),
                main_maker: "none".to_string(),
                timeout: 0,
            },
            &[],
            "Satellite label",
            None,
        )
        .unwrap_err();
    assert_eq!(
        "Timeout must be within limits (60 <= timeout <= 600)",
        err.root_cause().to_string()
    );

    let satellite_addr = app
        .instantiate_contract(
            satellite_code,
            owner.clone(),
            &InstantiateMsg {
                owner: owner.to_string(),
                astro_denom: "none".to_string(),
                transfer_channel: "none".to_string(),
                main_controller: "none".to_string(),
                main_maker: "none".to_string(),
                timeout: 60,
            },
            &[],
            "Satellite label",
            None,
        )
        .unwrap();

    let noop_code = app.store_code(noop_contract());
    let noop_addr = app
        .instantiate_contract(noop_code, owner.clone(), &Empty {}, &[], "none", None)
        .unwrap();

    let messages: Vec<_> = (0..5)
        .into_iter()
        .map(|_| wasm_execute(&noop_addr, &Empty {}, vec![]).unwrap().into())
        .collect();

    let err = app
        .execute_contract(
            owner.clone(),
            satellite_addr.clone(),
            &ExecuteMsg::CheckMessages(messages),
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::MessagesCheckPassed {},
        err.downcast().unwrap()
    );
}

#[test]
fn test_check_update_configs() {
    let owner = Addr::unchecked("owner");
    let mut app = mock_app(&owner, vec![]);

    let satellite_code = app.store_code(satellite_contract());
    let satellite_addr = app
        .instantiate_contract(
            satellite_code,
            owner.clone(),
            &InstantiateMsg {
                owner: owner.to_string(),
                astro_denom: "none".to_string(),
                transfer_channel: "none".to_string(),
                main_controller: "none".to_string(),
                main_maker: "none".to_string(),
                timeout: 60,
            },
            &[],
            "Satellite label",
            None,
        )
        .unwrap();

    app.execute_contract(
        owner.clone(),
        satellite_addr.clone(),
        &ExecuteMsg::UpdateConfig(UpdateConfigMsg {
            astro_denom: None,
            gov_channel: None,
            main_controller_addr: Some(Addr::unchecked("controller_addr_test").to_string()),
            main_maker: None,
            transfer_channel: None,
            accept_new_connections: None,
            timeout: None,
        }),
        &[],
    )
    .unwrap();

    if let Some(res) = app
        .wrap()
        .query_wasm_raw(satellite_addr.clone(), b"config".as_slice())
        .unwrap()
    {
        let res: Config = from_slice(&res).unwrap();
        assert_eq!("wasm.controller_addr_test", res.main_controller_port);
    }

    let err = app
        .execute_contract(
            owner.clone(),
            satellite_addr.clone(),
            &ExecuteMsg::UpdateConfig(UpdateConfigMsg {
                astro_denom: None,
                gov_channel: Some(Addr::unchecked("controller_addr_test").to_string()),
                main_controller_addr: None,
                main_maker: None,
                transfer_channel: None,
                accept_new_connections: Some(true),
                timeout: None,
            }),
            &[],
        )
        .unwrap_err();
    assert_eq!("The gov_channel and the accept_new_connections settings cannot be specified at the same time", err.root_cause().to_string());

    app.execute_contract(
        owner.clone(),
        satellite_addr.clone(),
        &ExecuteMsg::UpdateConfig(UpdateConfigMsg {
            astro_denom: None,
            gov_channel: Some(Addr::unchecked("controller_addr_test").to_string()),
            main_controller_addr: None,
            main_maker: None,
            transfer_channel: None,
            accept_new_connections: Some(false),
            timeout: None,
        }),
        &[],
    )
    .unwrap();
}
