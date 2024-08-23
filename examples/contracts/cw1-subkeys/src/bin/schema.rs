use cw1_subkeys::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
use sylvia::cw_schema::write_api;
use sylvia::cw_std::Empty;

#[cfg(not(tarpaulin_include))]
fn main() {
    write_api! {
        crate_name: sylvia::cw_schema,
        instantiate: InstantiateMsg,
        execute: ContractExecMsg<Empty, Empty>,
        query: ContractQueryMsg<Empty, Empty>,
    }
}
