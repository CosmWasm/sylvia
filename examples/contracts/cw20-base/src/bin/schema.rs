use cosmwasm_schema::write_api;

use cw20_base::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
use cw20_base::contract::{SvCustomMsg, SvCustomQuery};

#[cfg(not(tarpaulin_include))]
fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg<SvCustomMsg, SvCustomQuery>,
        query: ContractQueryMsg<SvCustomMsg, SvCustomQuery>,
    }
}
