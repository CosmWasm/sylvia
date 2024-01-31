use convert_case::{Case, Casing};
use proc_macro2::{Punct, TokenStream};
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use syn::fold::Fold;
use syn::parse::{Error, Nothing, Parse, ParseBuffer, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parenthesized, parse_quote, Attribute, GenericArgument, Ident, ImplItem, ImplItemFn, ItemImpl,
    ItemTrait, Path, PathArguments, Result, Token, TraitItem, Type,
};

use crate::crate_module;
use crate::strip_generics::StripGenerics;

/// Parsed arguments for `contract` macro
pub struct ContractArgs {
    /// Module in which contract impl block is defined.
    /// Used only while implementing `Interface` on `Contract`.
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

/// Parsed arguments for `entry_points` macro
pub struct EntryPointArgs {
    /// Types used in place of contracts generics.
    pub generics: Option<Punctuated<GenericArgument, Token![,]>>,
}

impl Parse for EntryPointArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Self { generics: None });
        }

        let path: Path = input.parse()?;

        let generics = match path.segments.last() {
            Some(segment) if segment.ident == "generics" => Some(extract_generics_from_path(&path)),
            _ => return Err(Error::new(path.span(), "Expected `generics`")),
        };

        let _: Nothing = input.parse()?;

        Ok(Self { generics })
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
    pub fn emit_ep_name(self) -> Ident {
        use MsgType::*;

        match self {
            Exec => parse_quote! { execute },
            Instantiate => parse_quote! { instantiate },
            Migrate => parse_quote! { migrate },
            Sudo => parse_quote! { sudo },
            Reply => parse_quote! { reply },
            Query => parse_quote! { query },
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

    pub fn emit_msg_name(&self, is_wrapper: bool) -> Type {
        match self {
            MsgType::Exec if is_wrapper => parse_quote! { ContractExecMsg },
            MsgType::Query if is_wrapper => parse_quote! { ContractQueryMsg },
            MsgType::Exec => parse_quote! { ExecMsg },
            MsgType::Query => parse_quote! { QueryMsg },
            MsgType::Instantiate => parse_quote! { InstantiateMsg },
            MsgType::Migrate => parse_quote! { MigrateMsg },
            MsgType::Reply => parse_quote! { ReplyMsg },
            MsgType::Sudo => todo!(),
        }
    }

    pub fn as_accessor_name(&self, is_wrapper: bool) -> Option<Type> {
        match self {
            MsgType::Exec if is_wrapper => Some(parse_quote! { ContractExec }),
            MsgType::Query if is_wrapper => Some(parse_quote! { ContractQuery }),
            MsgType::Instantiate => Some(parse_quote! { Instantiate }),
            MsgType::Exec => Some(parse_quote! { Exec }),
            MsgType::Query => Some(parse_quote! { Query }),
            MsgType::Migrate => Some(parse_quote! { Migrate }),
            MsgType::Sudo => Some(parse_quote! { Sudo }),
            MsgType::Reply => Some(parse_quote! { Reply }),
        }
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
        } else {
            Err(Error::new(
                input.span(),
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
        input.parse().map(|error| Self { error })
    }
}

#[derive(Debug)]
pub struct Customs {
    pub has_msg: bool,
    pub has_query: bool,
}

#[derive(Debug)]
pub struct ContractMessageAttr {
    pub module: Path,
    pub variant: Ident,
    pub customs: Customs,
    pub generics: Punctuated<GenericArgument, Token![,]>,
}

fn interface_has_custom(content: ParseStream) -> Result<Customs> {
    let mut customs = Customs {
        has_msg: false,
        has_query: false,
    };

    if !content.peek(Token![:]) {
        return Ok(customs);
    }

    let _: Token![:] = content.parse()?;
    let attr: Ident = content.parse()?;
    if attr != "custom" {
        return Ok(customs);
    }

    let custom_content;
    parenthesized!(custom_content in content);

    while !custom_content.is_empty() {
        let custom = custom_content.parse::<Path>()?;
        match custom.get_ident() {
            Some(ident) if ident == "msg" => customs.has_msg = true,
            Some(ident) if ident == "query" => customs.has_query = true,
            _ => {
                return Err(Error::new(
                    custom.span(),
                    "Invalid custom attribute, expected one of: `msg`, `query`",
                ))
            }
        }
        if !custom_content.peek(Token![,]) {
            break;
        }
        let _: Token![,] = custom_content.parse()?;
    }
    Ok(customs)
}

fn extract_generics_from_path(module: &Path) -> Punctuated<GenericArgument, Token![,]> {
    let generics = module.segments.last().map(|segment| {
        match segment.arguments.clone(){
            PathArguments::AngleBracketed(generics) => {
                generics.args
            },
            PathArguments::None => Default::default(),
            PathArguments::Parenthesized(generics) => {
                emit_error!(
                    generics.span(), "Found paranthesis wrapping generics in `messages` attribute.";
                    note = "Expected `messages` attribute to be in form `#[messages(Path<generics> as Type)]`"
                );
               Default::default()
            }
        }
    }).unwrap_or_default();

    generics
}

#[cfg(not(tarpaulin_include))]
// False negative. It is being called in closure
impl Parse for ContractMessageAttr {
    fn parse(input: ParseStream) -> Result<Self> {
        let module = input.parse()?;
        let generics = extract_generics_from_path(&module);
        let module = StripGenerics.fold_path(module);

        let variant = if input.parse::<Token![as]>().is_ok() {
            let variant: Ident = input.parse()?;
            if Some(variant.to_string())
                == module
                    .segments
                    .last()
                    .map(|name| name.ident.to_string().to_case(Case::UpperCamel))
            {
                emit_warning!(
                    variant.span(), "Redundant `as {}`.", variant;
                    note = "Interface name is a camel case version of the path and can be auto deduced."
                )
            }
            variant
        } else if let Some(module_name) = &module.segments.last() {
            let interface_name = module_name.ident.to_string().to_case(Case::UpperCamel);
            syn::Ident::new(&interface_name, module.span())
        } else {
            Ident::new("", module.span())
        };
        let customs = interface_has_custom(input)?;
        if !input.is_empty() {
            return Err(Error::new(
                input.span(),
                "Unexpected token on the end of `messages` attribtue",
            ));
        }
        Ok(Self {
            module,
            variant,
            customs,
            generics,
        })
    }
}

pub fn parse_struct_message(source: &ItemImpl, ty: MsgType) -> Option<(&ImplItemFn, MsgAttr)> {
    let mut methods = source.items.iter().filter_map(|item| match item {
        ImplItem::Fn(method) => {
            let msg_attr = method
                .attrs
                .iter()
                .find(|attr| attr.path().is_ident("msg"))?;

            let attr = match msg_attr
                .meta
                .require_list()
                .and_then(|meta| MsgAttr::parse.parse2(meta.tokens.clone()))
            {
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
                let custom = match attr
                    .meta
                    .require_list()
                    .and_then(|meta| Custom::parse.parse2(meta.tokens.clone()))
                {
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
        let mut custom = Self::default();

        while !input.is_empty() {
            let ty: Ident = input.parse()?;
            let _: Token![=] = input.parse()?;
            if ty == "msg" {
                custom.msg = Some(input.parse()?)
            } else if ty == "query" {
                custom.query = Some(input.parse()?)
            } else {
                return Err(Error::new(
                    ty.span(),
                    "Invalid custom type. Expected msg or query",
                ));
            };
            if !input.peek(Token![,]) {
                break;
            }
            let _: Token![,] = input.parse()?;
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
            ..
        } = self;

        let sylvia = crate_module();
        let values = msg_type.emit_ctx_values();

        quote! {
            #entry_point ( #values .into(), #sylvia ::cw_std::from_json::< #msg_name >(&msg)?)
                .map_err(Into::into)
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
            .filter_map(|attr| {
                match attr
                    .meta
                    .require_list()
                    .and_then(|meta| OverrideEntryPoint::parse.parse2(meta.tokens.clone()))
                {
                    Ok(entry_point) => Some(entry_point),
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        None
                    }
                }
            })
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
        let ty: Ident = input.parse()?;
        let _: Token![=] = input.parse()?;
        let entry_point = input.parse()?;

        let msg_content;
        parenthesized!(msg_content in input);

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
    if attr.path().segments.len() == 2 && attr.path().segments[0].ident == "sv" {
        Some(&attr.path().segments[1].ident)
    } else {
        None
    }
}
