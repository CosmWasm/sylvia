use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use syn::parse::{Error, Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Result, Token};

use super::MsgType;

pub struct VariantAttrForwarding {
    pub attrs: TokenStream,
}

impl VariantAttrForwarding {
    pub fn new(attr: &Attribute) -> Result<Self> {
        attr.meta
            .require_list()
            .and_then(|meta| VariantAttrForwarding::parse.parse2(meta.tokens.clone()))
            .map_err(|err| {
                emit_error!(attr.span(), err);
                err
            })
    }
}

impl Parse for VariantAttrForwarding {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.parse()?;
        Ok(Self { attrs })
    }
}

pub struct MsgAttrForwarding {
    pub msg_type: MsgType,
    pub attrs: TokenStream,
}

impl MsgAttrForwarding {
    pub fn new(attr: &Attribute) -> Result<Self> {
        attr.meta
            .require_list()
            .and_then(|meta| MsgAttrForwarding::parse.parse2(meta.tokens.clone()))
            .map_err(|err| {
                emit_error!(attr.span(), err);
                err
            })
    }
}

impl Parse for MsgAttrForwarding {
    fn parse(input: ParseStream) -> Result<Self> {
        let msg_type: Ident = input.parse()?;
        let _: Token![,] = input.parse()?;
        let attrs = input.parse()?;
        let msg_type = match msg_type.to_string().as_str() {
            "exec" => MsgType::Exec,
            "query" => MsgType::Query,
            "instantiate" => MsgType::Instantiate,
            "migrate" => MsgType::Migrate,
            "reply" => MsgType::Reply,
            "sudo" => MsgType::Sudo,
            _ => return Err(Error::new(
                input.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `migrate`, `reply` or `sudo`.",
            ))
        };
        Ok(Self { msg_type, attrs })
    }
}
