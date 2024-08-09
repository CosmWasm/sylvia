use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::{parse_quote, Error, Ident, MetaList, Result, Token, Type};

use crate::crate_module;

/// Type wrapping data parsed from `sv::custom` attribute.
#[derive(Debug, Default)]
pub struct Custom {
    pub msg: Option<Type>,
    pub query: Option<Type>,
}

impl Custom {
    pub fn new(attr: &MetaList) -> Result<Self> {
        Custom::parse.parse2(attr.tokens.clone()).map_err(|err| {
            emit_error!(err.span(), err);
            err
        })
    }

    pub fn msg_or_default(&self) -> Type {
        self.msg.clone().unwrap_or_else(Self::default_type)
    }

    pub fn query_or_default(&self) -> Type {
        self.query.clone().unwrap_or_else(Self::default_type)
    }

    pub fn default_type() -> Type {
        let sylvia = crate_module();
        parse_quote! { #sylvia ::cw_std::Empty }
    }
}

impl Parse for Custom {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut custom = Self::default();

        while !input.is_empty() {
            let ty: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match ty.to_string().as_str() {
                "msg" => custom.msg = Some(input.parse()?),
                "query" => custom.query = Some(input.parse()?),
                _ => {
                    return Err(Error::new(
                        ty.span(),
                        "Invalid custom type.\n
  = note: Expected `#[sv::custom(msg=SomeMsg, query=SomeQuery)]`.\n",
                    ))
                }
            }
            if !input.peek(Token![,]) {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        Ok(custom)
    }
}
