use cosmwasm_schema::write_api;

#[cfg(not(tarpaulin_include))]
fn main() {
    use generics_forwarded::contract::sv::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};
    use sylvia::types::SvCustomMsg;

    write_api! {
        instantiate: InstantiateMsg<SvCustomMsg>,
        // We should rethink this. We generate messages generic over types used in the methods.
        // To do so we however have to merge generics from the contract and interfaces methods.
        // This cause randomization of generics and user has to basically expand the macro
        // to see on which place given generic is placed.
        // Randomization is because f.e. for `<T1, T2, T3, T4>` depending which are used by
        // interfaces and which by contract we may end up with `ContractExecMsg<T4, T1, T3>`.
        //
        // ContractApi cannot be used here directly as `write_api` "expects string literal" here.
        // This potentially could be done with some type alias, not sure how it would affect the
        // schema.
        // execute: <GenericForwardedContract as ContractApi>::ContractExec,
        execute: ContractExecMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg>,
        query: ContractQueryMsg<SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg, SvCustomMsg>,
    }
}
