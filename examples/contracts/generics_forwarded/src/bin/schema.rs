use cosmwasm_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use generics_forwarded::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
    use sylvia::types::{SvCustomMsg, SvCustomQuery};

    write_api! {
        instantiate: InstantiateMsg<SvCustomMsg>,
        execute: ContractExecMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg,SvCustomQuery>,
        query: ContractQueryMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg,SvCustomQuery>,
    }
}
