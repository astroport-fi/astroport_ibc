use std::fmt::Debug;

use anyhow::Result as AnyResult;
use astro_satellite_package::{ExecuteMsg, InstantiateMsg, UpdateConfigMsg};
use astroport_ibc::{SIGNAL_OUTAGE_LIMITS, TIMEOUT_LIMITS};
use cosmwasm_std::{Addr, Api, CustomQuery, Storage};
use cw_multi_test::{
    AppResponse, Bank, ContractWrapper, Distribution, Executor, Gov, Ibc, Module, Staking,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;

use crate::{astroport_address, WKApp, ASTROPORT};

pub fn store_code<B, A, S, C, X, D, I, G>(app: &WKApp<B, A, S, C, X, D, I, G>) -> u64
where
    B: Bank,
    A: Api,
    S: Storage,
    C: Module,
    X: Staking,
    D: Distribution,
    I: Ibc,
    G: Gov,
    C::ExecT: Clone + Debug + PartialEq + JsonSchema + DeserializeOwned + 'static,
    C::QueryT: CustomQuery + DeserializeOwned + 'static,
{
    use astro_satellite as cnt;
    let contract = Box::new(ContractWrapper::new_with_empty(
        cnt::contract::execute,
        cnt::contract::instantiate,
        cnt::contract::query,
    ));

    app.borrow_mut().store_code(contract)
}

pub struct MockSatelliteBuilder<B, A, S, C: Module, X, D, I, G> {
    pub app: WKApp<B, A, S, C, X, D, I, G>,
    pub emergency_owner: Addr,
}

impl<B, A, S, C, X, D, I, G> MockSatelliteBuilder<B, A, S, C, X, D, I, G>
where
    B: Bank,
    A: Api,
    S: Storage,
    C: Module,
    X: Staking,
    D: Distribution,
    I: Ibc,
    G: Gov,
    C::ExecT: Clone + Debug + PartialEq + JsonSchema + DeserializeOwned + 'static,
    C::QueryT: CustomQuery + DeserializeOwned + 'static,
{
    pub fn new(app: &WKApp<B, A, S, C, X, D, I, G>, emergency_owner: &Addr) -> Self {
        Self {
            app: app.clone(),
            emergency_owner: emergency_owner.to_owned(),
        }
    }

    pub fn instantiate(self) -> MockSatellite<B, A, S, C, X, D, I, G> {
        let code_id = store_code(&self.app);
        let astroport = astroport_address();

        let address = self
            .app
            .borrow_mut()
            .instantiate_contract(
                code_id,
                astroport,
                &InstantiateMsg {
                    owner: ASTROPORT.to_owned(),
                    timeout: *TIMEOUT_LIMITS.start(),
                    emergency_owner: self.emergency_owner.to_string(),
                    max_signal_outage: *SIGNAL_OUTAGE_LIMITS.start(),
                    // The following parameters don't make sense with the current cw-multi-test
                    // version which doesn't support IBC properly
                    main_maker: "maker".to_owned(),
                    astro_denom: "astro".to_owned(),
                    main_controller: "controller".to_owned(),
                    transfer_channel: "channel".to_owned(),
                },
                &[],
                "Astroport Satellite",
                Some(ASTROPORT.to_owned()),
            )
            .unwrap();

        MockSatellite {
            app: self.app,
            address,
        }
    }
}

pub struct MockSatellite<B, A, S, C: Module, X, D, I, G> {
    pub app: WKApp<B, A, S, C, X, D, I, G>,
    pub address: Addr,
}

impl<B, A, S, C, X, D, I, G> MockSatellite<B, A, S, C, X, D, I, G>
where
    B: Bank,
    A: Api,
    S: Storage,
    C: Module,
    X: Staking,
    D: Distribution,
    I: Ibc,
    G: Gov,
    C::ExecT: Clone + Debug + PartialEq + JsonSchema + DeserializeOwned + 'static,
    C::QueryT: CustomQuery + DeserializeOwned + 'static,
{
    pub fn update_emergency_owner(
        &self,
        sender: &Addr,
        emergency_owner: &Addr,
    ) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            sender.to_owned(),
            self.address.clone(),
            &ExecuteMsg::UpdateConfig(UpdateConfigMsg {
                astro_denom: None,
                gov_channel: None,
                main_controller_addr: None,
                main_maker: None,
                transfer_channel: None,
                accept_new_connections: None,
                timeout: None,
                max_signal_outage: None,
                emergency_owner: Some(emergency_owner.to_string()),
            }),
            &[],
        )
    }

    pub fn update_admin(&self, sender: &Addr) -> AnyResult<AppResponse> {
        self.app.borrow_mut().execute_contract(
            sender.to_owned(),
            self.address.clone(),
            &ExecuteMsg::SetEmergencyOwnerAsAdmin {},
            &[],
        )
    }
}
