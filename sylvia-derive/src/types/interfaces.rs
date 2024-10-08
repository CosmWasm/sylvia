use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{ItemImpl, Path, Type};

use crate::crate_module;
use crate::parser::attributes::msg::MsgType;
use crate::parser::{ContractMessageAttr, ParsedSylviaAttributes};

/// Wrapper around [ContractMessageAttr] vector.
#[derive(Debug, Default)]
pub struct Interfaces {
    interfaces: Vec<ContractMessageAttr>,
}

impl Interfaces {
    pub fn new(source: &ItemImpl) -> Self {
        let interfaces = ParsedSylviaAttributes::new(source.attrs.iter()).messages_attrs;
        Self { interfaces }
    }

    pub fn emit_glue_message_variants(
        &self,
        msg_ty: &MsgType,
        contract: &Type,
    ) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, variant, ..
                } = interface;

                let interface_enum = quote! { < #contract as #module ::sv::InterfaceMessagesApi> };
                let type_name = msg_ty.as_accessor_name();

                quote! { #variant ( #interface_enum :: #type_name) }
            })
            .collect()
    }

    pub fn emit_glue_message_types(&self, msg_ty: &MsgType, contract: &Type) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, .. } = interface;

                let interface_enum = quote! { < #contract as #module ::sv::InterfaceMessagesApi> };
                let type_name = msg_ty.as_accessor_name();

                quote! { #interface_enum :: #type_name }
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

    pub fn emit_response_schemas_calls(
        &self,
        msg_ty: &MsgType,
        contract: &Type,
    ) -> Vec<TokenStream> {
        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr {
                    module, ..
                } = interface;

                let type_name = msg_ty.as_accessor_name();
                quote! {
                    <#contract as #module ::sv::InterfaceMessagesApi> :: #type_name :: response_schemas_impl()
                }
            })
            .collect()
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
                    #contract_enum_name:: #variant(msg) => #sylvia ::into_response::IntoResponse::into_response(msg.dispatch(contract, Into::into( #ctx ))?).map_err(Into::into)
                },
                _ => quote! {
                    #contract_enum_name :: #variant(msg) => msg.dispatch(contract, Into::into( #ctx ))
                },
            }
        }).collect()
    }

    pub fn variants_names(&self) -> impl Iterator<Item = &Ident> {
        self.interfaces.iter().map(|interface| &interface.variant)
    }

    pub fn variants_modules(&self) -> impl Iterator<Item = &Path> {
        self.interfaces.iter().map(|interface| &interface.module)
    }
}
