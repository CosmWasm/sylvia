use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    spanned::Spanned,
    GenericParam, ImplItem, ItemImpl, ItemTrait, TraitItem,
};

use crate::{
    check_generics::CheckGenerics,
    crate_module,
    message::{MsgField, MsgVariant},
    parser::{MsgAttr, MsgType},
};

pub struct Querier<'a> {
    variants: Vec<MsgVariant<'a>>,
}

impl<'a> Querier<'a> {
    pub fn for_contract(source: &'a ItemImpl, generics: &[&'a GenericParam]) -> Self {
        let mut generics_checker = CheckGenerics::new(generics);
        let variants: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Method(method) => {
                    let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                    let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                        Ok(attr) => attr,
                        Err(err) => {
                            emit_error!(method.span(), err);
                            return None;
                        }
                    };

                    if attr == MsgType::Query {
                        Some(MsgVariant::new(&method.sig, &mut generics_checker, attr))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();
        Self { variants }
    }

    pub fn for_interface(source: &'a ItemTrait, generics: &[&'a GenericParam]) -> Self {
        let mut generics_checker = CheckGenerics::new(generics);
        let variants: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Method(method) => {
                    let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                    let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                        Ok(attr) => attr,
                        Err(err) => {
                            emit_error!(method.span(), err);
                            return None;
                        }
                    };

                    if attr == MsgType::Query {
                        Some(MsgVariant::new(&method.sig, &mut generics_checker, attr))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();
        Self { variants }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self { variants } = self;

        let methods_impl = variants.iter().map(|variant| {
            let MsgVariant { name, fields, return_type, .. } = variant;

            let parameters = fields.iter().map(MsgField::emit_without_attrs);
            let fields_names = fields.iter().map(MsgField::name);
            let variant_name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

            quote! {
                fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError> {
                    let query = QueryMsg:: #variant_name (#(#fields_names),*);
                    self.querier.query_wasm_smart(self.contract, &query)
                }
            }
        });

        let methods_declaration = variants.iter().map(|variant| {
            let MsgVariant { name, fields, return_type, .. } = variant;

            let parameters = fields.iter().map(MsgField::emit_without_attrs);
            let variant_name = Ident::new(&name.to_string().to_case(Case::Snake), name.span());

            quote! {
                fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError>;}});

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
