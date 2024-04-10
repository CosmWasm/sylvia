use proc_macro_error::emit_error;
use syn::parse::{Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{parse_quote, Attribute, Ident, Result, Token, Type};

use crate::crate_module;

#[derive(Debug, Default)]
pub struct Custom {
    pub msg: Option<Type>,
    pub query: Option<Type>,
}

impl Custom {
    pub fn new(attr: &Attribute) -> Result<Self> {
        attr.meta
            .require_list()
            .and_then(|meta| Custom::parse.parse2(meta.tokens.clone()))
            .map_err(|err| {
                emit_error!(attr.span(), err);
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
            if ty == "msg" {
                custom.msg = Some(input.parse()?)
            } else if ty == "query" {
                custom.query = Some(input.parse()?)
            } else {
                emit_error!(ty.span(), "Invalid custom type.";
                    note = ty.span() => "Expected `#[sv::custom(msg=SomeMsg, query=SomeQuery)]`"
                );
            };
            if !input.peek(Token![,]) {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        Ok(custom)
    }
}
