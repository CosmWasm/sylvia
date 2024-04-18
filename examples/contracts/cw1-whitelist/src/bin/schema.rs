use cosmwasm_schema::write_api;

use cw1_whitelist::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

#[cfg(not(tarpaulin_include))]
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg<cw1_whitelist::contract::Cw1WhitelistContract>,
        query: ContractQueryMsg<cw1_whitelist::contract::Cw1WhitelistContract>,
    }
}
