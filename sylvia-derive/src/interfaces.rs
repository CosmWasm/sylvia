use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{ItemImpl, Path, Type};

use crate::crate_module;
use crate::parser::{ContractMessageAttr, MsgType};

#[derive(Debug, Default)]
pub struct Interfaces {
    interfaces: Vec<ContractMessageAttr>,
}

impl Interfaces {
    fn merge_module_with_name(message_attr: &ContractMessageAttr, name: &syn::Ident) -> syn::Ident {
        // ContractMessageAttr will fail to parse empty `#[messsages()]` attribute so we can safely unwrap here
        let syn::PathSegment { ident, .. } = &message_attr.module.segments.last().unwrap();
        let module_name = ident.to_string().to_case(Case::UpperCamel);
        syn::Ident::new(&format!("{}{}", module_name, name), name.span())
    }

    pub fn new(source: &ItemImpl) -> Self {
        let interfaces: Vec<_> = source
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("messages"))
            .filter_map(|attr| {
                let interface = match ContractMessageAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(interface) => interface,
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        return None;
                    }
                };

                Some(interface)
            })
            .collect();

        Self { interfaces }
    }

    pub fn interfaces(&self) -> &[ContractMessageAttr] {
        &self.interfaces
    }

    pub fn emit_querier_from_impl(&self) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.as_modules()
            .map(|module| {
                quote! {
                    impl<'a, C: #sylvia ::cw_std::CustomQuery> From<&'a BoundQuerier<'a, C>> for #module ::BoundQuerier<'a, C> {
                        fn from(querier: &'a BoundQuerier<'a, C>) -> Self {
                            Self::borrowed(querier.contract(),  querier.querier())
                        }
                    }
                }
            })
            .collect()
    }

    pub fn emit_proxy_accessors(&self, mt_app: &Type) -> Vec<TokenStream> {
        self.as_modules()
            .map(|module| {
                // ContractMessageAttr will fail to parse empty `#[messsages()]` attribute so we can safely unwrap here
                let module_name = &module.segments.last().unwrap().ident;
                let method_name = Ident::new(&format!("{}_proxy", module_name), module_name.span());
                let proxy_name = Ident::new(
                    &format!("{}Proxy", module_name.to_string().to_case(Case::UpperCamel)),
                    module_name.span(),
                );

                quote! {
                    pub fn #method_name (&self) -> #module ::trait_utils:: #proxy_name <'app, #mt_app> {
                        #module ::trait_utils:: #proxy_name ::new(self.contract_addr.clone(), self.app)
                    }
                }
            })
            .collect()
    }

    pub fn emit_glue_message_variants(
        &self,
        msg_ty: &MsgType,
        msg_name: &Ident,
    ) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module,
                    exec_generic_params,
                    query_generic_params,
                    variant,
                    ..
                } = interface;

                let generics = match msg_ty {
                    MsgType::Exec => exec_generic_params.as_slice(),
                    MsgType::Query => query_generic_params.as_slice(),
                    _ => &[],
                };

                let enum_name = Self::merge_module_with_name(interface, msg_name);
                quote! { #variant(#module :: #enum_name<#(#generics,)*>) }
            })
            .collect()
    }

    pub fn emit_messages_call(&self, msg_name: &Ident) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let enum_name = Self::merge_module_with_name(interface, msg_name);
                let module = &interface.module;
                quote! { &#module :: #enum_name :: messages()}
            })
            .collect()
    }

    pub fn emit_deserialization_attempts(&self, msg_name: &Ident) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, variant, ..
                } = interface;
                let enum_name = Self::merge_module_with_name(interface, msg_name);

                quote! {
                    let msgs = &#module :: #enum_name ::messages();
                    if msgs.into_iter().any(|msg| msg == &recv_msg_name) {
                        match val.deserialize_into() {
                            Ok(msg) => return Ok(Self:: #variant (msg)),
                            Err(err) => return Err(D::Error::custom(err)).map(Self:: #variant),
                        };
                    }
                }
            })
            .collect()
    }

    pub fn emit_response_schemas_calls(&self, msg_name: &Ident) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let enum_name = Self::merge_module_with_name(interface, msg_name);
                let module = &interface.module;
                quote! { #module :: #enum_name :: response_schemas_impl()}
            })
            .collect()
    }

    pub fn as_modules(&self) -> impl Iterator<Item = &Path> {
        self.interfaces.iter().map(|interface| &interface.module)
    }
}
