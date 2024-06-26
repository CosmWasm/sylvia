use proc_macro_error::emit_error;
use syn::parse::{Error, Parse, ParseStream, Parser};
use syn::{Ident, MetaList, Result, Token};

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

#[derive(Default)]
struct ArgumentParser {
    pub resp_type: Option<Ident>,
}

impl Parse for ArgumentParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut result = Self::default();
        while input.peek2(Ident) {
            let _: Token![,] = input.parse()?;
            let arg_type: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            match arg_type.to_string().as_str() {
                "resp" => {
                    let resp_type: Ident = input.parse()?;
                    result.resp_type = Some(resp_type);
                }
                _ => {
                    return Err(Error::new(
                        input.span(),
                        "Invalid argument type, expected `resp` or no argument.",
                    ))
                }
            }
        }
        Ok(result)
    }
}

/// Parsed representation of `#[sv::msg(...)]` attribute.
#[derive(Clone)]
pub enum MsgAttr {
    Exec,
    Query { resp_type: Option<Ident> },
    Instantiate,
    Migrate,
    Reply,
    Sudo,
}

impl MsgAttr {
    pub fn new(attr: &MetaList) -> Result<Self> {
        MsgAttr::parse.parse2(attr.tokens.clone()).map_err(|err| {
            emit_error!(err.span(), err);
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
    pub fn msg_type(&self) -> MsgType {
        match self {
            Self::Exec { .. } => MsgType::Exec,
            Self::Query { .. } => MsgType::Query,
            Self::Instantiate { .. } => MsgType::Instantiate,
            Self::Migrate { .. } => MsgType::Migrate,
            Self::Reply { .. } => MsgType::Reply,
            Self::Sudo { .. } => MsgType::Sudo,
        }
    }
}

impl Parse for MsgAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let ty: Ident = input.parse()?;
        let ArgumentParser { resp_type } = ArgumentParser::parse(input)?;

        let result = match ty.to_string().as_str() {
            "exec" => Self::Exec,
            "query" => Self::Query { resp_type },
            "instantiate" => Self::Instantiate,
            "migrate" => Self::Migrate,
            "reply" => Self::Reply,
            "sudo" => Self::Sudo,
            _ => return Err(Error::new(
                input.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `migrate`, `reply` or `sudo`.",
            ))
        };
        Ok(result)
    }
}
