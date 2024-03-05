use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, Result, Type};

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
    pub fn new(attr: &Attribute) -> Result<Self> {
        attr.meta
            .require_list()
            .and_then(|meta| ContractErrorAttr::parse.parse2(meta.tokens.clone()))
            .map_err(|err| {
                emit_error!(attr.span(), err);
                err
            })
    }
}

impl Parse for ContractErrorAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse().map(|error| Self { error })
    }
}
