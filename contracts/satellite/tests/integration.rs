use astro_ibc::astroport_governance::assembly::ProposalMessage;
use astro_ibc::satellite::{ExecuteMsg, InstantiateMsg, UpdateConfigMsg};
use astro_satellite::contract::{execute, instantiate, query, reply};
use astro_satellite::error::ContractError;
use cosmwasm_std::{
    wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response,
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

    fn noop_query(_deps: Deps, _env: Env, _info: MessageInfo, _msg: Empty) -> StdResult<Binary> {
        Ok(Default::default())
    }

    Box::new(ContractWrapper::new_with_empty(
        noop_execute,
        noop_execute,
        query,
    ))
}

fn proposal_msg(order: u64, msg: CosmosMsg) -> ProposalMessage {
    ProposalMessage {
        order: order.into(),
        msg,
    }
}

#[test]
fn test_check_messages() {
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
                timeout: 0,
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
        .map(|i| {
            proposal_msg(
                i,
                wasm_execute(&noop_addr, &Empty {}, vec![]).unwrap().into(),
            )
        })
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
    )
}
