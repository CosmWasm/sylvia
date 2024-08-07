use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;
use cw1_subkeys::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

#[cfg(not(tarpaulin_include))]
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg<Empty, Empty>,
        query: ContractQueryMsg<Empty, Empty>,
    }
}
