use std::cell::RefCell;
use std::rc::Rc;

use astro_satellite::contract::{execute, instantiate, query, reply};
use astro_satellite::error::ContractError;
use astro_satellite::state::Config;
use astro_satellite_package::{ExecuteMsg, InstantiateMsg, UpdateConfigMsg};
use astroport_mocks::{astroport_address, MockSatelliteBuilder};
use cosmwasm_std::{
    from_slice, wasm_execute, Addr, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, WasmMsg,
};

use astroport_ibc::{SIGNAL_OUTAGE_LIMITS, TIMEOUT_LIMITS};
use astroport_mocks::{
    anyhow::Result as AnyResult,
    cw_multi_test::{App, AppResponse, BasicApp, Contract, ContractWrapper, Executor},
};

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
                max_signal_outage: 1209600,
                emergency_owner: owner.to_string(),
            },
            &[],
            "Satellite label",
            None,
        )
        .unwrap_err();
    assert_eq!(
        format!(
            "Timeout must be within limits ({} <= timeout <= {})",
            TIMEOUT_LIMITS.start(),
            TIMEOUT_LIMITS.end()
        ),
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
                max_signal_outage: 1209600,
                emergency_owner: owner.to_string(),
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
            &ExecuteMsg::<Empty>::CheckMessages(messages),
            &[],
        )
        .unwrap_err();
    assert_eq!(
        ContractError::MessagesCheckPassed {},
        err.downcast().unwrap()
    );
}

#[test]
fn test_execute_multisig() {
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
                max_signal_outage: 1209600,
                emergency_owner: owner.to_string(),
            },
            &[],
            "Satellite label",
            Some(owner.to_string()),
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

    let random = Addr::unchecked("random");
    let err = app
        .execute_contract(
            random.clone(),
            satellite_addr.clone(),
            &ExecuteMsg::<Empty>::ExecuteFromMultisig(messages.clone()),
            &[],
        )
        .unwrap_err();
    assert_eq!(ContractError::Unauthorized {}, err.downcast().unwrap());

    app.execute_contract(
        owner.clone(),
        satellite_addr.clone(),
        &ExecuteMsg::<Empty>::ExecuteFromMultisig(messages),
        &[],
    )
    .unwrap();
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
                max_signal_outage: 1209600,
                emergency_owner: owner.to_string(),
            },
            &[],
            "Satellite label",
            None,
        )
        .unwrap();

    app.execute_contract(
        owner.clone(),
        satellite_addr.clone(),
        &ExecuteMsg::<Empty>::UpdateConfig(UpdateConfigMsg {
            astro_denom: None,
            gov_channel: None,
            main_controller_addr: Some(Addr::unchecked("controller_addr_test").to_string()),
            main_maker: None,
            transfer_channel: None,
            accept_new_connections: None,
            timeout: None,
            emergency_owner: None,
            max_signal_outage: None,
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
            &ExecuteMsg::<Empty>::UpdateConfig(UpdateConfigMsg {
                astro_denom: None,
                gov_channel: Some(Addr::unchecked("controller_addr_test").to_string()),
                main_controller_addr: None,
                main_maker: None,
                transfer_channel: None,
                accept_new_connections: Some(true),
                timeout: None,
                max_signal_outage: None,
                emergency_owner: None,
            }),
            &[],
        )
        .unwrap_err();
    assert_eq!("The gov_channel and the accept_new_connections settings cannot be specified at the same time", err.root_cause().to_string());

    app.execute_contract(
        owner.clone(),
        satellite_addr.clone(),
        &ExecuteMsg::<Empty>::UpdateConfig(UpdateConfigMsg {
            astro_denom: None,
            gov_channel: Some(Addr::unchecked("controller_addr_test").to_string()),
            main_controller_addr: None,
            main_maker: None,
            transfer_channel: None,
            accept_new_connections: Some(false),
            timeout: None,
            max_signal_outage: None,
            emergency_owner: None,
        }),
        &[],
    )
    .unwrap();
}

#[test]
fn check_ownership_capabilities() {
    let app = Rc::new(RefCell::new(BasicApp::default()));

    let astroport = astroport_address();
    let emergency_owner = Addr::unchecked("emergency_owner");
    let another_user = Addr::unchecked("another_user");

    let satellite = MockSatelliteBuilder::new(&app, &emergency_owner).instantiate();

    // Setting the same emergency owner just to check the capability
    satellite
        .update_emergency_owner(&astroport, &emergency_owner)
        .unwrap();

    // As the owner may manage this contract, using this endpoint does not make sense
    assert_unauthorized(satellite.update_admin(&astroport));

    // Instead, we set the contract's admin to itself
    app.borrow_mut()
        .execute(
            astroport.clone(),
            WasmMsg::UpdateAdmin {
                contract_addr: satellite.address.to_string(),
                admin: satellite.address.to_string(),
            }
            .into(),
        )
        .unwrap();

    // The emergency owner may not change any parameters nor contract admin until max signal outage
    // is reached
    assert_unauthorized(satellite.update_emergency_owner(&emergency_owner, &emergency_owner));
    assert_unauthorized(satellite.update_admin(&emergency_owner));

    // No one else, of course, may change anything
    assert_unauthorized(satellite.update_emergency_owner(&another_user, &another_user));
    assert_unauthorized(satellite.update_admin(&another_user));

    // Let's check the same when signal outage is reached
    app.borrow_mut().update_block(|b| {
        b.time = b.time.plus_seconds(*SIGNAL_OUTAGE_LIMITS.start() + 1);
        b.height += 1;
    });

    // The main owner still can change everything
    satellite
        .update_emergency_owner(&astroport, &emergency_owner)
        .unwrap();

    // Again, as the owner may manage this contract, using this endpoint does not make sense
    assert_unauthorized(satellite.update_admin(&astroport));

    // The emergency owner may change many parameters or contract admin now
    satellite
        .update_emergency_owner(&emergency_owner, &emergency_owner)
        .unwrap();
    satellite.update_admin(&emergency_owner).unwrap();

    // Again, no one else, of course, may change anything
    assert_unauthorized(satellite.update_emergency_owner(&another_user, &another_user));
    assert_unauthorized(satellite.update_admin(&another_user));
}

fn assert_unauthorized(result: AnyResult<AppResponse>) {
    assert_eq!(
        result.unwrap_err().downcast::<ContractError>().unwrap(),
        ContractError::Unauthorized {}
    )
}
