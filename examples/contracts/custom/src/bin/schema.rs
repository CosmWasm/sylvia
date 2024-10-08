use sylvia::cw_schema::write_api;

use custom::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        crate_name: sylvia::cw_schema,
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
