use proc_macro2::{Punct, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::parse::{Error, Nothing, Parse, ParseBuffer, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse_quote, Attribute, Ident, ImplItem, ImplItemMethod, ItemImpl, Path, Result,
    Token, Type,
};

use crate::crate_module;

/// Parser arguments for `contract` macro
pub struct ContractArgs {
    /// Module name wrapping generated messages, by default no additional module is created
    pub module: Option<Path>,
}

impl Parse for ContractArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut module = None;

        while !input.is_empty() {
            let attr: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            if attr == "module" {
                module = Some(input.parse()?);
            } else {
                return Err(Error::new(attr.span(), "expected `module`"));
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else if !input.is_empty() {
                return Err(input.error("Unexpected token, comma expected"));
            }
        }

        let _: Nothing = input.parse()?;

        Ok(ContractArgs { module })
    }
}

/// Type of message to be generated
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MsgType {
    Exec,
    Query,
    Instantiate,
    Migrate,
    Reply,
}

/// `#[msg(...)]` attribute for `interface` macro
pub enum MsgAttr {
    Exec,
    Query { resp_type: Option<Ident> },
    Instantiate { name: Ident },
    Migrate { name: Ident },
    Reply,
}

impl MsgType {
    pub fn emit_ctx_type(self) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                (#sylvia ::cw_std::DepsMut, #sylvia ::cw_std::Env, #sylvia ::cw_std::MessageInfo)
            },
            Migrate | Reply => quote! {
                (#sylvia ::cw_std::DepsMut, #sylvia ::cw_std::Env)
            },
            Query => quote! {
                (#sylvia ::cw_std::Deps, #sylvia ::cw_std::Env)
            },
        }
    }

    /// Emits type which should be returned by dispatch function for this kind of message
    pub fn emit_result_type(self, msg_type: impl ToTokens, err_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate | Migrate | Reply => {
                quote! {
                    std::result::Result< #sylvia:: cw_std::Response <#msg_type>, #err_type>
                }
            }
            Query => quote! {
                std::result::Result<#sylvia ::cw_std::Binary, #err_type>
            },
        }
    }
}

impl PartialEq<MsgType> for MsgAttr {
    fn eq(&self, other: &MsgType) -> bool {
        self.msg_type() == *other
    }
}

impl MsgAttr {
    fn parse_query(content: ParseBuffer) -> Result<Self> {
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
        use MsgAttr::*;

        match self {
            Exec => MsgType::Exec,
            Query { .. } => MsgType::Query,
            Instantiate { .. } => MsgType::Instantiate,
            Migrate { .. } => MsgType::Migrate,
            Reply => MsgType::Reply,
        }
    }
}

impl Parse for MsgAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let ty: Ident = content.parse()?;
        if ty == "exec" {
            Ok(Self::Exec)
        } else if ty == "query" {
            Self::parse_query(content)
        } else if ty == "instantiate" {
            let name = Ident::new("InstantiateMsg", content.span());
            Ok(Self::Instantiate { name })
        } else if ty == "migrate" {
            let name = Ident::new("MigrateMsg", content.span());
            Ok(Self::Migrate { name })
        } else if ty == "reply" {
            Ok(Self::Reply)
        } else {
            Err(Error::new(
                ty.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `migrate`",
            ))
        }
    }
}

#[derive(Debug)]
pub struct ContractErrorAttr {
    pub error: Type,
}

#[cfg(not(tarpaulin_include))]
// False negative. It is being called in closure
impl Parse for ContractErrorAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        content.parse().map(|error| Self { error })
    }
}

#[derive(Debug)]
pub struct ContractMessageAttr {
    pub module: Path,
    pub exec_generic_params: Vec<Path>,
    pub query_generic_params: Vec<Path>,
    pub variant: Ident,
    pub has_custom_msg: bool,
}

#[cfg(not(tarpaulin_include))]
// False negative. Called in function below
fn parse_generics(content: &ParseBuffer) -> Result<Vec<Path>> {
    let _: Token![<] = content.parse()?;
    let mut params = vec![];

    loop {
        let param: Path = content.parse()?;
        params.push(param);

        let generics_close: Option<Token![>]> = content.parse()?;
        if generics_close.is_some() {
            break;
        }

        let comma: Option<Token![,]> = content.parse()?;
        if comma.is_none() {
            return Err(Error::new(content.span(), "Expected comma or `>`"));
        }
    }

    Ok(params)
}

#[cfg(not(tarpaulin_include))]
// False negative. It is being called in closure
impl Parse for ContractMessageAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let module = content.parse()?;

        let generics_open: Option<Token![:]> = content.parse()?;
        let mut exec_generic_params = vec![];
        let mut query_generic_params = vec![];
        let mut has_custom_msg = false;

        if generics_open.is_some() {
            loop {
                let ty: Ident = content.parse()?;
                let params = if ty == "exec" {
                    &mut exec_generic_params
                } else if ty == "query" {
                    &mut query_generic_params
                } else {
                    return Err(Error::new(ty.span(), "Invalid message type"));
                };

                *params = parse_generics(&content)?;

                if content.peek(Token![as]) {
                    break;
                }

                let _: Token![,] = content.parse()?;
            }
        }

        let _: Token![as] = content.parse()?;
        let variant = content.parse()?;

        if content.peek(Token![:]) {
            let _: Token![:] = content.parse()?;
            let attr: Ident = content.parse()?;
            if attr == "custom" {
                let custom_content;
                parenthesized!(custom_content in content);
                let custom = custom_content.parse::<Path>()?;
                has_custom_msg = custom.is_ident("msg");
            }
        }

        if !content.is_empty() {
            return Err(Error::new(
                content.span(),
                "Unexpected token on the end of `message` attribtue",
            ));
        }

        Ok(Self {
            module,
            exec_generic_params,
            query_generic_params,
            variant,
            has_custom_msg,
        })
    }
}

pub fn parse_struct_message(source: &ItemImpl, ty: MsgType) -> Option<(&ImplItemMethod, MsgAttr)> {
    let mut methods = source.items.iter().filter_map(|item| match item {
        ImplItem::Method(method) => {
            let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
            let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                Ok(attr) => attr,
                Err(err) => {
                    emit_error!(method.span(), err);
                    return None;
                }
            };

            if attr == ty {
                Some((method, attr))
            } else {
                None
            }
        }
        _ => None,
    });

    let (method, msg_attr) = if let Some(method) = methods.next() {
        method
    } else {
        if ty == MsgType::Instantiate {
            emit_error!(source.span(), "No instantiation message");
        }
        return None;
    };

    if let Some((obsolete, _)) = methods.next() {
        emit_error!(
            obsolete.span(), "More than one instantiation or migration message";
            note = method.span() => "Instantiation/Migration message previously defined here"
        );
    }
    Some((method, msg_attr))
}

#[derive(Debug)]
pub struct Custom<'a> {
    msg: Path,
    _query: Path,
    input_attr: Option<&'a Attribute>,
}

impl Default for Custom<'_> {
    fn default() -> Self {
        let sylvia = crate_module();

        Self {
            msg: parse_quote!(#sylvia ::cw_std::Empty),
            _query: parse_quote!(#sylvia ::cw_std::Empty),
            input_attr: None,
        }
    }
}

impl<'a> Custom<'a> {
    pub fn new(attrs: &'a [Attribute]) -> Self {
        let mut customs = attrs
            .iter()
            .filter(|attr| match sylvia_attribute(attr) {
                Some(attr) => attr == "custom",
                None => false,
            })
            .filter_map(|attr| {
                let custom = match Custom::parse.parse2(attr.tokens.clone()) {
                    Ok(mut custom) => {
                        custom.input_attr = Some(attr);
                        custom
                    }
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        return None;
                    }
                };

                Some(custom)
            });

        let custom = customs.next().unwrap_or_default();

        for redefined in customs {
            let redefined = redefined.input_attr.unwrap();
            emit_error!(
              redefined, "The attribute `custom` is redefined";
              note = custom.input_attr.span() => "Previous definition of the attribute `custom`";
              note = "Only one `custom` attribute can exist on a single sylvia entity"
            );
        }

        custom
    }

    pub fn msg(&self) -> &Path {
        &self.msg
    }
}

#[cfg(not(tarpaulin_include))]
// False negative. It is being called in closure
impl Parse for Custom<'_> {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let mut custom = Self::default();

        while !content.is_empty() {
            let ty: Ident = content.parse()?;
            let _: Token![=] = content.parse()?;
            if ty == "msg" {
                custom.msg = content.parse()?
            } else if ty == "query" {
                custom._query = content.parse()?
            } else {
                return Err(Error::new(
                    ty.span(),
                    "Invalid custom type. Expected msg or query",
                ));
            };
            if !content.peek(Token![,]) {
                break;
            }
            let _: Token![,] = content.parse()?;
        }

        Ok(custom)
    }
}

pub fn sylvia_attribute(attr: &Attribute) -> Option<&Ident> {
    if attr.path.segments.len() == 2 && attr.path.segments[0].ident == "sv" {
        Some(&attr.path.segments[1].ident)
    } else {
        None
    }
}
