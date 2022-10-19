use cosmwasm_schema::write_api;
use cw20_ics20_orig::msg::{ExecuteMsg, InitMsg, QueryMsg};

fn main() {
    write_api! {
        instantiate: InitMsg,
        query: QueryMsg,
        execute: ExecuteMsg,
    }
}
