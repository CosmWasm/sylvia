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

    pub fn emit_glue_message_variants(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module,
                    variant,
                    generics,
                    ..
                } = interface;
                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };

                let interface_enum =
                    quote! { <#module ::InterfaceTypes #generics as #sylvia ::types::InterfaceMessages> };
                if msg_ty == &MsgType::Query {
                    quote! { #variant ( #interface_enum :: Query) }
                } else {
                    quote! { #variant ( #interface_enum :: Exec)}
                }
            })
            .collect()
    }

    pub fn emit_messages_call(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, generics, ..
                } = interface;
                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };
                let type_name = msg_ty.as_accessor_name();
                quote! {
                    &<#module :: InterfaceTypes #generics as #sylvia ::types::InterfaceMessages> :: #type_name :: messages()
                }
            })
            .collect()
    }

    pub fn emit_deserialization_attempts(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, variant, generics, ..
                } = interface;
                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };

                let type_name = msg_ty.as_accessor_name();
                quote! {
                    let msgs = &<#module :: InterfaceTypes #generics as #sylvia ::types::InterfaceMessages> :: #type_name :: messages();
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

    pub fn emit_response_schemas_calls(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, generics, ..
                } = interface;

                let generics = if !generics.is_empty() {
                    quote! { < #generics > }
                } else {
                    quote! {}
                };

                let type_name = msg_ty.as_accessor_name();
                quote! {
                    <#module :: InterfaceTypes #generics as #sylvia ::types::InterfaceMessages> :: #type_name :: response_schemas_impl()
                }
            })
            .collect()
    }

    pub fn as_modules(&self) -> impl Iterator<Item = &Path> {
        self.interfaces.iter().map(|interface| &interface.module)
    }

    pub fn get_only_interface(&self) -> Option<&ContractMessageAttr> {
        let interfaces = &self.interfaces;
        match interfaces.len() {
            0 => None,
            1 => Some(&interfaces[0]),
            _ => {
                let first = &interfaces[0];
                for redefined in &interfaces[1..] {
                    emit_error!(
                      redefined.module, "The attribute `messages` is redefined";
                      note = first.module.span() => "Previous definition of the attribute `messsages`";
                      note = "Only one `messages` attribute can exist on an interface implementation on contract"
                    );
                }
                None
            }
        }
    }
}
