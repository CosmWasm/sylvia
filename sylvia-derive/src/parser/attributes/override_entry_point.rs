use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Error, Parse, ParseStream, Parser};
use syn::{parenthesized, Ident, MetaList, Path, Result, Token, Type};

use crate::crate_module;
use crate::parser::MsgType;

#[derive(Debug, Clone)]
pub struct OverrideEntryPoint {
    entry_point: Path,
    msg_name: Type,
    msg_type: MsgType,
}

impl OverrideEntryPoint {
    pub fn new(attr: &MetaList) -> Result<Self> {
        OverrideEntryPoint::parse
            .parse2(attr.tokens.clone())
            .map_err(|err| {
                emit_error!(err.span(), err);
                err
            })
    }

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

pub trait FilteredOverrideEntryPoints {
    fn get_entry_point(&self, ty: MsgType) -> Option<&OverrideEntryPoint>;
}

impl FilteredOverrideEntryPoints for &[OverrideEntryPoint] {
    fn get_entry_point(&self, ty: MsgType) -> Option<&OverrideEntryPoint> {
        self.iter().find(|entry_point| entry_point.msg_type == ty)
    }
}

impl FilteredOverrideEntryPoints for &Vec<OverrideEntryPoint> {
    fn get_entry_point(&self, ty: MsgType) -> Option<&OverrideEntryPoint> {
        self.iter().find(|entry_point| entry_point.msg_type == ty)
    }
}

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
