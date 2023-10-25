use cosmwasm_schema::write_api;

use entry_points_overriding::contract::sv::{ContractQueryMsg, InstantiateMsg};
use entry_points_overriding::messages::CustomExecMsg;
use entry_points_overriding::messages::SudoMsg;

#[cfg(not(tarpaulin_include))]
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: CustomExecMsg,
        query: ContractQueryMsg,
        sudo: SudoMsg
    }
}
