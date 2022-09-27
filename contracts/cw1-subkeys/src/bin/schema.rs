use cosmwasm_schema::write_api;

use cw1_subkeys::contract::{ContractExecMsg, ContractQueryMsg};

use cw1_whitelist::contract::InstantiateMsg;

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
