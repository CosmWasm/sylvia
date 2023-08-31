use proc_macro2::{Punct, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Error, Nothing, Parse, ParseBuffer, ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse_quote, Attribute, Ident, ImplItem, ImplItemMethod, ItemImpl, ItemTrait,
    Path, Result, Token, TraitItem, Type,
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
    Sudo,
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
    pub fn emit_ctx_type(self, query_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                (#sylvia ::cw_std::DepsMut< #query_type >, #sylvia ::cw_std::Env, #sylvia ::cw_std::MessageInfo)
            },
            Migrate | Reply | Sudo => quote! {
                (#sylvia ::cw_std::DepsMut< #query_type >, #sylvia ::cw_std::Env)
            },
            Query => quote! {
                (#sylvia ::cw_std::Deps< #query_type >, #sylvia ::cw_std::Env)
            },
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn emit_ctx_params(self, query_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                deps: #sylvia ::cw_std::DepsMut< #query_type>, env: #sylvia ::cw_std::Env, info: #sylvia ::cw_std::MessageInfo
            },
            Migrate | Reply | Sudo => quote! {
                deps: #sylvia ::cw_std::DepsMut< #query_type>, env: #sylvia ::cw_std::Env
            },
            Query => quote! {
                deps: #sylvia ::cw_std::Deps< #query_type>, env: #sylvia ::cw_std::Env
            },
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn emit_ep_name(self) -> TokenStream {
        use MsgType::*;

        match self {
            Exec => quote! { execute },
            Instantiate => quote! { instantiate },
            Migrate => quote! { migrate },
            Sudo => quote! { sudo },
            Reply => quote! { reply },
            Query => quote! { query },
        }
    }

    pub fn emit_ctx_values(self) -> TokenStream {
        use MsgType::*;

        match self {
            Exec | Instantiate => quote! { deps, env, info },
            Migrate | Reply | Query | Sudo => quote! { deps, env },
        }
    }

    /// Emits type which should be returned by dispatch function for this kind of message
    pub fn emit_result_type(self, msg_type: &Type, err_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate | Migrate | Reply | Sudo => {
                quote! {
                    std::result::Result< #sylvia:: cw_std::Response <#msg_type>, #err_type>
                }
            }
            Query => quote! {
                std::result::Result<#sylvia ::cw_std::Binary, #err_type>
            },
        }
    }

    pub fn emit_msg_name(&self) -> Type {
        match self {
            MsgType::Exec => parse_quote! { ContractExecMsg },
            MsgType::Query => parse_quote! { ContractQueryMsg },
            MsgType::Instantiate => parse_quote! { InstantiateMsg },
            MsgType::Migrate => parse_quote! { MigrateMsg },
            MsgType::Reply => parse_quote! { ReplyMsg },
            MsgType::Sudo => todo!(),
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
    pub has_custom_query: bool,
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

fn interface_has_custom(content: ParseStream) -> Result<(bool, bool)> {
    let mut has_custom_msg = false;
    let mut has_custom_query = false;

    let _: Token![:] = content.parse()?;
    let attr: Ident = content.parse()?;
    if attr != "custom" {
        return Ok((has_custom_msg, has_custom_query));
    }

    let custom_content;
    parenthesized!(custom_content in content);

    while !custom_content.is_empty() {
        let custom = custom_content.parse::<Path>()?;
        match custom.get_ident() {
            Some(ident) if ident == "msg" => has_custom_msg = true,
            Some(ident) if ident == "query" => has_custom_query = true,
            _ => {
                return Err(Error::new(
                    custom.span(),
                    "Invalid custom attribute, expected one of: `msg`, `query`",
                ))
            }
        }
    }
    Ok((has_custom_msg, has_custom_query))
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

        let (has_custom_msg, has_custom_query) = if content.peek(Token![:]) {
            interface_has_custom(&content)?
        } else {
            (false, false)
        };

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
            has_custom_query,
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

pub fn parse_associated_custom_type(source: &ItemTrait, type_name: &str) -> Option<Type> {
    let trait_name = &source.ident;
    source.items.iter().find_map(|item| match item {
        TraitItem::Type(ty) if ty.ident == type_name => {
            let type_name = Ident::new(type_name, ty.span());
            Some(parse_quote! { <C as #trait_name>:: #type_name})
        }
        _ => None,
    })
}

#[derive(Debug, Default)]
pub struct Custom<'a> {
    msg: Option<Type>,
    query: Option<Type>,
    input_attr: Option<&'a Attribute>,
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

    pub fn msg_or_default(&self) -> Type {
        self.msg.clone().unwrap_or_else(Self::default_type)
    }

    pub fn query_or_default(&self) -> Type {
        self.query.clone().unwrap_or_else(Self::default_type)
    }

    pub fn msg(&self) -> Option<Type> {
        self.msg.clone()
    }

    pub fn query(&self) -> Option<Type> {
        self.query.clone()
    }

    pub fn default_type() -> Type {
        let sylvia = crate_module();
        parse_quote! { #sylvia ::cw_std::Empty }
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
                custom.msg = Some(content.parse()?)
            } else if ty == "query" {
                custom.query = Some(content.parse()?)
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

#[derive(Debug)]
pub struct OverrideEntryPoint {
    entry_point: Path,
    msg_name: Type,
    msg_type: MsgType,
}

impl OverrideEntryPoint {
    pub fn emit_multitest_dispatch(&self) -> TokenStream {
        let Self {
            entry_point,
            msg_name,
            msg_type,
        } = self;

        let sylvia = crate_module();
        let values = msg_type.emit_ctx_values();

        quote! {
            #entry_point ( #values .into(), #sylvia ::cw_std::from_slice::< #msg_name >(&msg)?)
                .map_err(Into::into)
        }
    }

    pub fn emit_multitest_default_dispatch(ty: MsgType) -> TokenStream {
        let sylvia = crate_module();

        let values = ty.emit_ctx_values();
        let msg_name = ty.emit_msg_name();

        quote! {
            #sylvia ::cw_std::from_slice::< #msg_name >(&msg)?
                .dispatch(self, ( #values ))
                .map_err(Into::into)
        }
    }

    #[cfg(not(tarpaulin_include))]
    pub fn emit_default_entry_point(
        custom_msg: &Type,
        custom_query: &Type,
        name: &Type,
        error: &Type,
        msg_type: MsgType,
    ) -> TokenStream {
        let sylvia = crate_module();

        let resp_type = match msg_type {
            MsgType::Query => quote! { #sylvia ::cw_std::Binary },
            _ => quote! { #sylvia ::cw_std::Response < #custom_msg > },
        };
        let params = msg_type.emit_ctx_params(custom_query);
        let values = msg_type.emit_ctx_values();
        let ep_name = msg_type.emit_ep_name();
        let msg_name = msg_type.emit_msg_name();

        quote! {
            #[#sylvia ::cw_std::entry_point]
            pub fn #ep_name (
                #params ,
                msg: #msg_name,
            ) -> Result<#resp_type, #error> {
                msg.dispatch(&#name ::new() , ( #values )).map_err(Into::into)
            }
        }
    }
}

#[derive(Debug)]
pub struct OverrideEntryPoints(Vec<OverrideEntryPoint>);

impl OverrideEntryPoints {
    pub fn new(attrs: &[Attribute]) -> Self {
        let entry_points = attrs
            .iter()
            .filter(|attr| match sylvia_attribute(attr) {
                Some(attr) => attr == "override_entry_point",
                None => false,
            })
            .filter_map(
                |attr| match OverrideEntryPoint::parse.parse2(attr.tokens.clone()) {
                    Ok(entry_point) => Some(entry_point),
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        None
                    }
                },
            )
            .collect();

        Self(entry_points)
    }

    pub fn get_entry_point(&self, ty: MsgType) -> Option<&OverrideEntryPoint> {
        self.0.iter().find(|entry_point| entry_point.msg_type == ty)
    }
}

#[cfg(not(tarpaulin_include))]
// False negative. It is being called in closure
impl Parse for OverrideEntryPoint {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);

        let ty: Ident = content.parse()?;
        let _: Token![=] = content.parse()?;
        let entry_point = content.parse()?;

        let msg_content;
        parenthesized!(msg_content in content);

        let msg_name = msg_content.parse()?;

        let msg_type = match ty.to_string().as_str() {
            "exec" =>  MsgType::Exec,
            "instantiate" =>  MsgType::Instantiate,
            "query" =>  MsgType::Instantiate,
            "migrate" => MsgType::Migrate,
            "reply" => MsgType::Reply,
            "sudo" =>  MsgType::Sudo,
            &_ => {
                return Err(Error::new(
                    ty.span(),
                    "Invalid entry point. Expected exec, instantiate, query, migrate, reply or sudo. Found {ty}",
                ))
            }
        };

        Ok(Self {
            entry_point,
            msg_name,
            msg_type,
        })
    }
}

pub fn sylvia_attribute(attr: &Attribute) -> Option<&Ident> {
    if attr.path.segments.len() == 2 && attr.path.segments[0].ident == "sv" {
        Some(&attr.path.segments[1].ident)
    } else {
        None
    }
}
