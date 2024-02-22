use proc_macro2::Punct;
use syn::parse::{Error, Parse, ParseBuffer, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Ident, Result};

use proc_macro_error::emit_error;

/// Type of message to be generated
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MsgType {
    Exec,
    Query,
    Instantiate,
    Migrate,
    Reply,
    Sudo,
}

/// Parsed representation of `#[sv::msg(...)]` attribute.
#[derive(Clone)]
pub enum MsgAttr {
    Exec,
    Query { resp_type: Option<Ident> },
    Instantiate { name: Ident },
    Migrate { name: Ident },
    Reply,
    Sudo,
}

impl MsgAttr {
    pub fn new(attr: &Attribute) -> Result<Self> {
        attr.meta
            .require_list()
            .and_then(|meta| MsgAttr::parse.parse2(meta.tokens.clone()))
            .map_err(|err| {
                emit_error!(attr.span(), err);
                err
            })
    }
}

impl PartialEq<MsgType> for MsgAttr {
    fn eq(&self, other: &MsgType) -> bool {
        self.msg_type() == *other
    }
}

impl MsgAttr {
    fn parse_query(content: &ParseBuffer) -> Result<Self> {
        if content.peek2(Ident) {
            let _: Punct = content.parse()?;
            let _: Ident = content.parse()?;
            let _: Punct = content.parse()?;
            let resp_type: Option<Ident> = Some(content.parse()?);
            Ok(Self::Query { resp_type })
        } else {
            Ok(Self::Query { resp_type: None })
        }
    }

    pub fn msg_type(&self) -> MsgType {
        MsgType::Exec.as_accessor_name();
        match self {
            Self::Exec => MsgType::Exec,
            Self::Query { .. } => MsgType::Query,
            Self::Instantiate { .. } => MsgType::Instantiate,
            Self::Migrate { .. } => MsgType::Migrate,
            Self::Reply => MsgType::Reply,
            Self::Sudo => MsgType::Sudo,
        }
    }
}

impl Parse for MsgAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Ident = input.parse()?;

        if ty == "exec" {
            Ok(Self::Exec)
        } else if ty == "query" {
            Self::parse_query(input)
        } else if ty == "instantiate" {
            let name = Ident::new("InstantiateMsg", input.span());
            Ok(Self::Instantiate { name })
        } else if ty == "migrate" {
            let name = Ident::new("MigrateMsg", input.span());
            Ok(Self::Migrate { name })
        } else if ty == "reply" {
            Ok(Self::Reply)
        } else if ty == "sudo" {
            Ok(Self::Sudo)
        } else {
            Err(Error::new(
                input.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `migrate`, `reply` or `sudo`.",
            ))
        }
    }
}
