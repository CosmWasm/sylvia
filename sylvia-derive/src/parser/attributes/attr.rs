use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use syn::parse::{Error, Parse, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Ident, MetaList, Result, Token};

use super::MsgType;

#[derive(Clone, Debug)]
pub struct VariantAttrForwarding {
    pub attrs: TokenStream,
    pub span: Span,
}

impl VariantAttrForwarding {
    pub fn new(attr: &MetaList) -> Self {
        VariantAttrForwarding {
            attrs: attr.tokens.clone(),
            span: attr.span(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MsgAttrForwarding {
    pub msg_type: MsgType,
    pub attrs: TokenStream,
}

impl MsgAttrForwarding {
    pub fn new(attr: &MetaList) -> Result<Self> {
        MsgAttrForwarding::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(attr.tokens.span(), err);
                err
            })
    }
}

impl Parse for MsgAttrForwarding {
    fn parse(input: ParseStream) -> Result<Self> {
        let error_msg =
            "Expected attribute of the form: `#[sv::msg_attr(msg_type, attribute_to_forward)]`";
        let msg_type: Ident = input
            .parse()
            .map_err(|err| Error::new(err.span(), error_msg))?;
        let _: Token![,] = input
            .parse()
            .map_err(|err| Error::new(err.span(), error_msg))?;
        let attrs: TokenStream = input
            .parse()
            .map_err(|err| Error::new(err.span(), error_msg))?;
        if attrs.is_empty() {
            return Err(Error::new(attrs.span(), error_msg));
        }
        let msg_type = match msg_type.to_string().as_str() {
            "exec" => MsgType::Exec,
            "query" => MsgType::Query,
            "instantiate" => MsgType::Instantiate,
            "migrate" => MsgType::Migrate,
            "reply" => MsgType::Reply,
            "sudo" => MsgType::Sudo,
            _ => return Err(Error::new(
                msg_type.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `migrate`, `reply` or `sudo`.",
            ))
        };
        Ok(Self { msg_type, attrs })
    }
}
