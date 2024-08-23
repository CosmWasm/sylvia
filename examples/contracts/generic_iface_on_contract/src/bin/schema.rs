use sylvia::cw_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use generic_iface_on_contract::contract::sv::{
        ContractExecMsg, ContractQueryMsg, InstantiateMsg,
    };

    write_api! {
        crate_name: sylvia::cw_schema,
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
