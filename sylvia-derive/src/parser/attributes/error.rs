use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{parse_quote, MetaList, Result, Type};

use crate::crate_module;

#[derive(Debug)]
pub struct ContractErrorAttr {
    pub error: Type,
}

impl Default for ContractErrorAttr {
    fn default() -> Self {
        let sylvia = crate_module();
        Self {
            error: parse_quote! { #sylvia ::cw_std::StdError },
        }
    }
}

impl ContractErrorAttr {
    pub fn new(attr: &MetaList) -> Result<Self> {
        ContractErrorAttr::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })
    }
}

impl Parse for ContractErrorAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(|error| Self { error })
    }
}
