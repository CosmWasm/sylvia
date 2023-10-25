use cosmwasm_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use generic_iface_on_contract::contract::sv::{
        ContractExecMsg, ContractQueryMsg, InstantiateMsg,
    };

    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
