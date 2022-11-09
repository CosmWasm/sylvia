use proc_macro2::{Punct, TokenStream};
use quote::quote;
use syn::parse::{Error, Nothing, Parse, ParseBuffer, ParseStream};
use syn::{parenthesized, parse_quote, Ident, Path, Result, Token, Type};

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
    pub module: Option<Ident>,
    /// The type of a contract error for entry points - `ContractError` by default
    pub error: Type,
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
                return Err(Error::new(attr.span(), "expected `module` or `msg_type`"));
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
        let mut error = parse_quote!(ContractError);

        while !input.is_empty() {
            let attr: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;

            if attr == "module" {
                module = Some(input.parse()?);
            } else if attr == "error" {
                error = input.parse()?;
            } else {
                return Err(Error::new(attr.span(), "expected `module` or `error`"));
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else if !input.is_empty() {
                return Err(input.error("Unexpected token, comma expected"));
            }
        }

        let _: Nothing = input.parse()?;

        Ok(ContractArgs { module, error })
    }
}

/// Type of message to be generated
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MsgType {
    Exec,
    Query,
    Instantiate,
    Reply,
    Migrate,
}

/// `#[msg(...)]` attribute for `interface` macro
pub enum MsgAttr {
    Exec,
    Query { resp_type: Option<Ident> },
    Instantiate { name: Ident },
    Reply,
    Migrate,
}

impl MsgType {
    pub fn emit_ctx_type(self) -> TokenStream {
        use MsgType::*;

        match self {
            Exec | Instantiate => quote! {
                (cosmwasm_std::DepsMut, cosmwasm_std::Env, cosmwasm_std::MessageInfo)
            },
            Migrate | Reply => quote! {
                (cosmwasm_std::DepsMut, cosmwasm_std::Env)
            },
            Query => quote! {
                (cosmwasm_std::Deps, cosmwasm_std::Env)
            },
        }
    }

    /// Emits type which should be returned by dispatch function for this kind of message
    pub fn emit_result_type(self, msg_type: &Option<Type>, err_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match (self, msg_type) {
            (Exec, Some(msg_type))
            | (Instantiate, Some(msg_type))
            | (Migrate, Some(msg_type))
            | (Reply, Some(msg_type)) => {
                quote! {
                    std::result::Result<#sylvia ::cw_std::Response<#msg_type>, #err_type>
                }
            }
            (Exec, None) | (Instantiate, None) | (Migrate, None) | (Reply, None) => quote! {
                std::result::Result<#sylvia ::cw_std::Response, #err_type>
            },

            (Query, _) => quote! {
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
    fn parse_instantiate(content: ParseBuffer) -> Result<Self> {
        let name = Ident::new("InstantiateMsg", content.span());

        Ok(Self::Instantiate { name })
    }

    fn parse_query(content: ParseBuffer) -> Result<Self> {
        if content.peek2(Ident) {
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
            Reply => MsgType::Reply,
            Migrate => MsgType::Migrate,
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
            Self::parse_instantiate(content)
        } else if ty == "reply" {
            Ok(Self::Reply)
        } else if ty == "migrate" {
            Ok(Self::Migrate)
        } else {
            Err(Error::new(
                ty.span(),
                "Invalid message type, expected one of: `exec`, `query`, `instantiate`, `reply`, `migrate`",
            ))
        }
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
