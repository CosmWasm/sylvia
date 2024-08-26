use sylvia::cw_schema::write_api;

fn main() {
    use generic_contract::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
    use generic_contract::contract::SvCustomMsg;

    write_api! {
        crate_name: sylvia::cw_schema,
        instantiate: InstantiateMsg<SvCustomMsg>,
        execute: ContractExecMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg,>,
        query: ContractQueryMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg,>,
    }
}
