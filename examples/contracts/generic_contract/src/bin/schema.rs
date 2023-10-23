use cosmwasm_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use generic_contract::contract::{
        ContractExecMsg, ContractQueryMsg, ExternalMsg, InstantiateMsg,
    };

    write_api! {
        instantiate: InstantiateMsg<ExternalMsg>,
        execute: ContractExecMsg<ExternalMsg>,
        query: ContractQueryMsg<ExternalMsg>,
    }
}
