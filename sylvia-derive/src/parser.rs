use proc_macro2::{Punct, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Error, Nothing, Parse, ParseBuffer, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{parenthesized, Ident, ImplItem, ImplItemMethod, ItemImpl, Path, Result, Token, Type};

use crate::crate_module;

/// Parsed arguments for `interface` macro
pub struct InterfaceArgs {
    /// Module name wrapping generated messages, by default no additional module is created
    pub module: Option<Ident>,
    /// The type being a parameter of `CosmosMsg` for blockchain it is intendet to be used; can be
    /// set to any of generic parameters to create interface being generic over blockchains; If not
    /// provided, cosmos messages would be unparametrized (so default would be used)
    pub msg_type: Option<Type>,
}

/// Parser arguments for `contract` macro
pub struct ContractArgs {
    /// Module name wrapping generated messages, by default no additional module is created
    pub module: Option<Path>,
}

impl Parse for InterfaceArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut module = None;
        let mut msg_type = None;

        while !input.is_empty() {
            let attr: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            if attr == "module" {
                module = Some(input.parse()?);
            } else if attr == "msg_type" {
                msg_type = Some(input.parse()?);
            } else {
                return Err(Error::new(attr.span(), "expected `module`, or `msg_type`"));
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else if !input.is_empty() {
                return Err(input.error("Unexpected token, comma expected"));
            }
        }

        let _: Nothing = input.parse()?;

        Ok(InterfaceArgs { module, msg_type })
    }
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
    pub fn emit_result_type(self, msg_type: &Option<Type>, err_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        #[cfg(not(tarpaulin_include))]
        let response_type = msg_type
            .as_ref()
            .map(|ty| quote! { #sylvia ::cw_std::Response< #ty > })
            .unwrap_or_else(|| quote! { #sylvia ::cw_std::Response });

        match self {
            Exec | Instantiate | Migrate | Reply => {
                quote! {
                    std::result::Result<#response_type, #err_type>
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
