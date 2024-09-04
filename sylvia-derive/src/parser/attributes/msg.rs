use proc_macro_error::emit_error;
use syn::parse::{Error, Parse, ParseStream, Parser};
use syn::{bracketed, Ident, MetaList, Result, Token};

/// Supported message types.
/// Representation of the first parameter in `#[sv::msg(..)] attribute.
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
    pub query_resp_type: Option<Ident>,
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
                    result.query_resp_type = Some(resp_type);
                }
                "handlers" => {
                    let _: Token![=] = input.parse()?;
                    let handlers_content;
                    bracketed!(handlers_content in input);

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
#[derive(Copy, Debug, Default, Clone, PartialEq)]
pub enum ReplyOn {
    Success,
    Failure,
    #[default]
    Always,
}

impl std::fmt::Display for ReplyOn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplyOn::Success => f.write_str("success"),
            ReplyOn::Failure => f.write_str("failure"),
            ReplyOn::Always => f.write_str("always"),
        }
    }
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

    /// Checks if two [ReplyOn] values are unique and if [ReplyOn::Always] is not one of them as it is exclusive
    /// and cannot be paired with any other value.
    pub fn excludes(&self, other: &Self) -> bool {
        let are_equal = self == other;
        let is_any_always = self == &ReplyOn::Always || other == &ReplyOn::Always;

        are_equal || is_any_always
    }
}

/// Parsed representation of `#[sv::msg(...)]` attribute.
#[derive(Debug, Clone)]
pub struct MsgAttr {
    msg_type: MsgType,
    query_resp_type: Option<Ident>,
    reply_handlers: Vec<Ident>,
    reply_on: ReplyOn,
}

impl MsgAttr {
    pub fn new(attr: &MetaList) -> Result<Self> {
        MsgAttr::parse.parse2(attr.tokens.clone()).map_err(|err| {
            emit_error!(err.span(), err);
            err
        })
    }

    pub fn msg_type(&self) -> MsgType {
        self.msg_type
    }

    pub fn resp_type(&self) -> &Option<Ident> {
        &self.query_resp_type
    }

    pub fn handlers(&self) -> &[Ident] {
        &self.reply_handlers
    }

    pub fn reply_on(&self) -> ReplyOn {
        self.reply_on
    }
}

impl PartialEq<MsgType> for MsgAttr {
    fn eq(&self, other: &MsgType) -> bool {
        self.msg_type() == *other
    }
}

impl Parse for MsgAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let msg_type: Ident = input.parse()?;
        let msg_type = MsgType::new(&msg_type)?;
        let ArgumentParser {
            query_resp_type,
            reply_handlers,
            reply_on,
        } = ArgumentParser::parse(input)?;

        Ok(Self {
            msg_type,
            query_resp_type,
            reply_handlers,
            reply_on: reply_on.unwrap_or_default(),
        })
    }
}
