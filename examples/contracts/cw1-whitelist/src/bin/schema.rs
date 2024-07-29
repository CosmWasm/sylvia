use cosmwasm_schema::write_api;
use cosmwasm_std::Empty;

use cw1_whitelist::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
use cw1_whitelist::contract::SvCustomQuery;

#[cfg(not(tarpaulin_include))]
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg<Empty, SvCustomQuery>,
        query: ContractQueryMsg<Empty, SvCustomQuery>,
    }
}
