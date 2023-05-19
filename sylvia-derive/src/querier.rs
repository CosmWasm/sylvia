use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::GenericParam;

use crate::check_generics::CheckGenerics;
use crate::crate_module;
use crate::message::MsgVariant;
use crate::parser::{MsgAttr, MsgType};
use crate::utils::MethodDataIterator;

pub struct Querier<'a> {
    variants: Vec<MsgVariant<'a>>,
}

impl<'a> Querier<'a> {
    pub fn new(source: MethodDataIterator<'a>, generics: &[&'a GenericParam]) -> Self {
        let mut generics_checker = CheckGenerics::new(generics);

        let variants: Vec<_> = source
            .filter_map(|(attrs, sig, span)| {
                let msg_attr = attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                    Ok(attr) => attr,
                    Err(err) => {
                        emit_error!(span, err);
                        return None;
                    }
                };

                if attr == MsgType::Query {
                    Some(MsgVariant::new(sig, &mut generics_checker, attr))
                } else {
                    None
                }
            })
            .collect();
        Self { variants }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self { variants } = self;

        let methods_impl = variants.iter().map(MsgVariant::emit_querier_impl);
        let methods_declaration = variants.iter().map(MsgVariant::emit_querier_declaration);

        quote! {
            pub struct BoundQuerier<'a, C: #sylvia ::cw_std::CustomQuery> {
                contract: &'a #sylvia ::cw_std::Addr,
                querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>,
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery> Querier for BoundQuerier<'a, C> {
                #(#methods_impl)*
            }


            pub trait Querier {
                #(#methods_declaration)*
            }
        }
    }
}
