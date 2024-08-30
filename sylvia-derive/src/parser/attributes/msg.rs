use proc_macro_error::emit_error;
use syn::parse::{Error, Parse, ParseStream, Parser};
use syn::{parenthesized, Ident, MetaList, Result, Token};

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

/// ArgumentParser holds `resp` parameter parsed from `sv::msg` attribute.
#[derive(Default)]
struct ArgumentParser {
    pub resp_type: Option<Ident>,
    pub reply_handlers: Vec<Ident>,
    pub reply_on: Option<ReplyOn>,
}

impl Parse for ArgumentParser {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut result = Self::default();

        while input.peek2(Ident) {
            let _: Token![,] = input.parse()?;
            let arg_type: Ident = input.parse()?;
            match arg_type.to_string().as_str() {
                "resp" => {
                    let _: Token![=] = input.parse()?;
                    let resp_type: Ident = input.parse()?;
                    result.resp_type = Some(resp_type);
                }
                "handlers" => {
                    let handlers_content;
                    parenthesized!(handlers_content in input);

                    while !handlers_content.is_empty() {
                        let handler = handlers_content.parse::<Ident>()?;
                        result.reply_handlers.push(handler);
                        if !handlers_content.peek(Token![,]) {
                            break;
                        }
                        let _: Token![,] = handlers_content.parse()?;
                    }
                }
                "reply_on" => {
                    let _: Token![=] = input.parse()?;
                    let reply_on: Ident = input.parse()?;
                    let reply_on = ReplyOn::new(reply_on)?;
                    result.reply_on = Some(reply_on);
                }
                _ => {
                    return Err(Error::new(
                        arg_type.span(),
                        "Invalid argument type, expected `resp`, `handlers`, `reply_on` or no argument.",
                    ))
                }
            }
        }
        Ok(result)
    }
}

/// Representation of `reply_on` parameter in `#[sv::msg(reply(...))]` attribute.
#[derive(Default, Clone)]
pub enum ReplyOn {
    Success,
    Failure,
    #[default]
    Always,
}

impl ReplyOn {
    pub fn new(reply_on: Ident) -> Result<Self> {
        match reply_on.to_string().as_str() {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            "always" => Ok(Self::Always),
            _ => Err(Error::new(
                reply_on.span(),
                "Invalid argument type, expected one of `success`, `failure` or `always`.",
            )),
        }
    }
}

/// Parsed representation of `#[sv::msg(...)]` attribute.
#[derive(Clone)]
pub enum MsgAttr {
    Exec,
    Query {
        resp_type: Option<Ident>,
    },
    Instantiate,
    Migrate,
    Reply {
        _handlers: Vec<Ident>,
        _reply_on: ReplyOn,
    },
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
        let ArgumentParser {
            resp_type,
            reply_handlers,
            reply_on,
        } = ArgumentParser::parse(input)?;

        let result = match ty.to_string().as_str() {
            "exec" => Self::Exec,
            "query" => Self::Query { resp_type },
            "instantiate" => Self::Instantiate,
            "migrate" => Self::Migrate,
            "reply" => Self::Reply {_handlers: reply_handlers, _reply_on: reply_on.unwrap_or_default()},
            "sudo" => Self::Sudo,
            _ => return Err(Error::new(
                input.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `migrate`, `reply` or `sudo`.",
            ))
        };
        Ok(result)
    }
}
