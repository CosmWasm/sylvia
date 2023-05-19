use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{parse_quote, GenericParam, Ident, ItemImpl, ItemTrait, TraitItem, Type};

use crate::crate_module;
use crate::message::{ContractEnumMessage, EnumMessage, GlueMessage, StructMessage};
use crate::multitest::{MultitestHelpers, TraitMultitestHelpers};
use crate::parser::{ContractArgs, ContractErrorAttr, InterfaceArgs, MsgType};
use crate::querier::Querier;
use crate::remote::Remote;
use crate::utils::Items;

/// Preprocessed `interface` macro input
pub struct TraitInput<'a> {
    attributes: &'a InterfaceArgs,
    item: &'a ItemTrait,
    generics: Vec<&'a GenericParam>,
}

/// Preprocessed `contract` macro input for non-trait impl block
pub struct ImplInput<'a> {
    attributes: &'a ContractArgs,
    error: Type,
    item: &'a ItemImpl,
    generics: Vec<&'a GenericParam>,
}

impl<'a> TraitInput<'a> {
    #[cfg(not(tarpaulin_include))]
    // This requires invalid implementation which would fail at compile time and making it impossible to test
    pub fn new(attributes: &'a InterfaceArgs, item: &'a ItemTrait) -> Self {
        let generics = item.generics.params.iter().collect();

        if !item
            .items
            .iter()
            .any(|item| matches!(item, TraitItem::Type(ty) if ty.ident == Ident::new("Error", ty.ident.span())))
        {
            emit_error!(
                item.ident.span(), "Missing `Error` type defined for trait.";
                note = "Error is an error type returned by generated types dispatch function. Messages handling function have to return an error type convertible to this Error type.";
                note = "A trait error type should be bound to implement `From<cosmwasm_std::StdError>`.";
            );
        }

        Self {
            attributes,
            item,
            generics,
        }
    }

    pub fn process(&self) -> TokenStream {
        let messages = self.emit_messages();
        let multitest_helpers = self.emit_helpers();
        let remote = Remote::for_interface().emit();
        let querier = Querier::new(self.item.items(), &self.generics).emit();

        if let Some(module) = &self.attributes.module {
            #[cfg(not(tarpaulin_include))]
            {
                quote! {
                    pub mod #module {
                        use super::*;

                        #messages

                        #multitest_helpers

                        #remote

                        #querier
                    }
                }
            }
        } else {
            #[cfg(not(tarpaulin_include))]
            {
                quote! {
                    #messages

                    #multitest_helpers

                    #remote

                    #querier
                }
            }
        }
    }

    fn emit_helpers(&self) -> TokenStream {
        if cfg!(feature = "mt") {
            let multitest_helpers = TraitMultitestHelpers::new(self.item);
            multitest_helpers.emit()
        } else {
            quote! {}
        }
    }

    fn emit_messages(&self) -> TokenStream {
        let exec = self.emit_msg(
            &Ident::new("ExecMsg", Span::mixed_site()),
            MsgType::Exec,
            self.attributes,
        );
        let query = self.emit_msg(
            &Ident::new("QueryMsg", Span::mixed_site()),
            MsgType::Query,
            self.attributes,
        );

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #exec

                #query
            }
        }
    }

    fn emit_msg(&self, name: &Ident, msg_ty: MsgType, args: &InterfaceArgs) -> TokenStream {
        EnumMessage::new(name, self.item, msg_ty, &self.generics, args).emit()
    }
}

impl<'a> ImplInput<'a> {
    pub fn new(attributes: &'a ContractArgs, item: &'a ItemImpl) -> Self {
        let sylvia = crate_module();

        let generics = item.generics.params.iter().collect();

        let error = item
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("error"))
            .and_then(
                |attr| match ContractErrorAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(error) => Some(error.error),
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        None
                    }
                },
            )
            .unwrap_or_else(|| parse_quote! { #sylvia ::cw_std::StdError });

        Self {
            attributes,
            item,
            generics,
            error,
        }
    }

    pub fn process(&self) -> TokenStream {
        let is_trait = self.item.trait_.is_some();

        let multitest_helpers = if cfg!(feature = "mt") {
            MultitestHelpers::new(self.item, is_trait, &self.error, &self.generics).emit()
        } else {
            quote! {}
        };

        if is_trait {
            return multitest_helpers;
        }

        let messages = self.emit_messages();
        let remote = Remote::for_contract(self.item).emit();
        let querier = Querier::new(self.item.items(), &self.generics).emit();

        #[cfg(not(tarpaulin_include))]
        let code = quote! {
            #messages

            #multitest_helpers

            #remote

            #querier
        };

        if let Some(module) = &self.attributes.module {
            #[cfg(not(tarpaulin_include))]
            {
                quote! {
                    pub mod #module {
                        use super::*;

                        #code
                    }
                }
            }
        } else {
            code
        }
    }

    fn emit_messages(&self) -> TokenStream {
        let instantiate = self.emit_struct_msg(MsgType::Instantiate);
        let migrate = self.emit_struct_msg(MsgType::Migrate);
        let exec_impl =
            self.emit_enum_msg(&Ident::new("ExecMsg", Span::mixed_site()), MsgType::Exec);
        let query_impl =
            self.emit_enum_msg(&Ident::new("QueryMsg", Span::mixed_site()), MsgType::Query);
        let exec = self.emit_glue_msg(&Ident::new("ExecMsg", Span::mixed_site()), MsgType::Exec);
        let query = self.emit_glue_msg(&Ident::new("QueryMsg", Span::mixed_site()), MsgType::Query);

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #instantiate

                #exec_impl

                #query_impl

                #migrate

                #exec

                #query
            }
        }
    }

    fn emit_struct_msg(&self, msg_ty: MsgType) -> TokenStream {
        StructMessage::new(self.item, msg_ty, &self.generics).map_or(quote! {}, |msg| msg.emit())
    }

    fn emit_enum_msg(&self, name: &Ident, msg_ty: MsgType) -> TokenStream {
        ContractEnumMessage::new(name, self.item, msg_ty, &self.generics, &self.error).emit()
    }

    fn emit_glue_msg(&self, name: &Ident, msg_ty: MsgType) -> TokenStream {
        GlueMessage::new(name, self.item, msg_ty, &self.error).emit()
    }
}
