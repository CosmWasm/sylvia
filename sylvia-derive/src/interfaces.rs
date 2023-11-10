use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{GenericArgument, GenericParam, ItemImpl};

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
            .filter(|attr| attr.path().is_ident("messages"))
            .filter_map(|attr| {
                let interface = match attr
                    .meta
                    .require_list()
                    .and_then(|meta| ContractMessageAttr::parse.parse2(meta.tokens.clone()))
                {
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

    pub fn emit_querier_from_impl(&self, contract_generics: &[&GenericParam]) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, generics ,..} = interface;
                quote! {
                    impl<'a, C: #sylvia ::cw_std::CustomQuery, #(#contract_generics,)* > From<&'a BoundQuerier<'a, C, #(#contract_generics,)* >> for #module ::sv::BoundQuerier<'a, C, #generics > {
                        fn from(querier: &'a BoundQuerier<'a, C, #(#contract_generics,)* >) -> Self {
                            Self::borrowed(querier.contract(),  querier.querier())
                        }
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
                    quote! { <#module ::sv::Api #generics as #sylvia ::types::InterfaceApi> };
                let type_name = msg_ty.as_accessor_name();

                quote! { #variant ( #interface_enum :: #type_name) }
            })
            .collect()
    }

    pub fn emit_messages_call(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, .. } = interface;

                let ep_name = msg_ty.emit_ep_name();
                let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), module.span());
                quote! {
                    &#module ::sv:: #messages_fn_name()
                }
            })
            .collect()
    }

    pub fn emit_deserialization_attempts(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, variant, ..
                } = interface;
                let ep_name = msg_ty.emit_ep_name();
                let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), module.span());

                quote! {
                    let msgs = &#module ::sv:: #messages_fn_name();
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
                    <#module ::sv::Api #generics as #sylvia ::types::InterfaceApi> :: #type_name :: response_schemas_impl()
                }
            })
            .collect()
    }

    pub fn emit_remote_from_impl(&self, contract_generics: &[&GenericParam]) -> Vec<TokenStream> {
        self.interfaces.iter().map(|interface| {
            let ContractMessageAttr {
                module, generics, ..
            } = interface;

            quote! {
                impl<'a, #(#contract_generics,)* > From<&'a Remote<'a, #(#contract_generics,)* >> for #module ::sv::Remote<'a, #generics > {
                    fn from(remote: &'a Remote<'a, #(#contract_generics,)* >) -> Self {
                        #module ::sv::Remote::borrowed(remote.as_ref())
                    }
                }
            }
        }).collect()
    }

    pub fn emit_dispatch_arms(&self, msg_ty: &MsgType) -> Vec<TokenStream> {
        let sylvia = crate_module();
        let contract_enum_name = msg_ty.emit_msg_wrapper_name();

        self.interfaces.iter().map(|interface| {
            let ContractMessageAttr {
                variant,
                customs,
                ..
            } = interface;

            let ctx = msg_ty.emit_ctx_dispatch_values(customs);

            match (msg_ty, customs.has_msg) {
                (MsgType::Exec, true) | (MsgType::Sudo, true) => quote! {
                    #contract_enum_name:: #variant(msg) => #sylvia ::into_response::IntoResponse::into_response(msg.dispatch(contract, Into::into( #ctx ))?)
                },
                _ => quote! {
                    #contract_enum_name :: #variant(msg) => msg.dispatch(contract, Into::into( #ctx ))
                },
            }
        }).collect()
    }

    pub fn as_generic_args(&self) -> Vec<&GenericArgument> {
        self.interfaces
            .iter()
            .flat_map(|interface| &interface.generics)
            .collect()
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
