#![cfg(not(tarpaulin_include))]

use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::Addr;

pub mod satellite;

pub const ASTROPORT: &str = "astroport";

pub fn astroport_address() -> Addr {
    Addr::unchecked(ASTROPORT)
}

pub use cw_multi_test;
use cw_multi_test::{App, Module, WasmKeeper};
pub use satellite::{MockSatellite, MockSatelliteBuilder};

pub type WKApp<B, A, S, C, X, D, I, G> = Rc<
    RefCell<App<B, A, S, C, WasmKeeper<<C as Module>::ExecT, <C as Module>::QueryT>, X, D, I, G>>,
>;
