use sylvia::cw_schema::write_api;

use entry_points_overriding::contract::sv::{ContractQueryMsg, InstantiateMsg};
use entry_points_overriding::messages::{CustomExecMsg, SudoMsg};

fn main() {
    write_api! {
        crate_name: sylvia::cw_schema,
        instantiate: InstantiateMsg,
        execute: CustomExecMsg,
        query: ContractQueryMsg,
        sudo: SudoMsg
    }
}
