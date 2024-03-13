use cosmwasm_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use generic_contract::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
    use generic_contract::contract::SvCustomMsg;

    write_api! {
        instantiate: InstantiateMsg<SvCustomMsg>,
        execute: ContractExecMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg>,
        query: ContractQueryMsg<SvCustomMsg>,
    }
}
