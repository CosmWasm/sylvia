use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::{GenericParam, Ident, ItemImpl, ItemTrait, TraitItem};

use crate::associated_types::{AssociatedTypes, ImplAssociatedTypes, ItemType};
use crate::interfaces::Interfaces;
use crate::message::{
    ContractApi, ContractEnumMessage, EnumMessage, GlueMessage, InterfaceApi, MsgVariants,
    StructMessage,
};
use crate::multitest::{ContractMtHelpers, ImplMtHelpers};
use crate::parser::{ContractArgs, ContractErrorAttr, Custom, MsgType, OverrideEntryPoints};
use crate::querier::{ContractQuerier, ImplQuerier, TraitQuerier};
use crate::remote::{ContractRemote, InterfaceRemote};
use crate::utils::is_trait;
use crate::variant_descs::AsVariantDescs;

/// Preprocessed `interface` macro input
pub struct TraitInput<'a> {
    item: &'a ItemTrait,
    custom: Custom<'a>,
    associated_types: AssociatedTypes<'a>,
}

/// Preprocessed `contract` macro input for non-trait impl block
pub struct ImplInput<'a> {
    attributes: &'a ContractArgs,
    error: ContractErrorAttr,
    item: &'a ItemImpl,
    generics: Vec<&'a GenericParam>,
    custom: Custom<'a>,
    override_entry_points: OverrideEntryPoints,
    interfaces: Interfaces,
}

impl<'a> TraitInput<'a> {
    #[cfg(not(tarpaulin_include))]
    pub fn new(item: &'a ItemTrait) -> Self {
        if !item.generics.params.is_empty() {
            emit_error!(
                item.ident.span(), "Generics on traits are not supported. Use associated types instead.";
                note = "Sylvia interfaces can be implemented only a single time per contract.";
            );
        }

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

        let custom = Custom::new(&item.attrs);
        let associated_types = AssociatedTypes::new(item);

        Self {
            item,
            custom,
            associated_types,
        }
    }

    pub fn process(&self) -> TokenStream {
        let Self {
            associated_types,
            item,
            custom,
        } = self;
        let messages = self.emit_messages();
        let remote = InterfaceRemote::new(associated_types).emit();
        let associated_names: Vec<_> = associated_types.as_names().collect();

        let query_variants =
            MsgVariants::new(item.as_variants(), MsgType::Query, &associated_names, &None);
        let querier = TraitQuerier::new(&query_variants, associated_types).emit_trait_querier();

        let interface_messages = InterfaceApi::new(item, associated_types, custom).emit();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod sv {
                    use super::*;
                    #messages

                    #remote

                    #querier

                    #interface_messages
                }
            }
        }
    }

    fn emit_messages(&self) -> TokenStream {
        let exec = self.emit_msg(MsgType::Exec);
        let query = self.emit_msg(MsgType::Query);
        let sudo = self.emit_msg(MsgType::Sudo);

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #exec

                #query

                #sudo
            }
        }
    }

    fn emit_msg(&self, msg_ty: MsgType) -> TokenStream {
        let where_clause = &self.associated_types.as_where_clause();
        let associated_names: Vec<_> = self
            .associated_types
            .without_error()
            .map(ItemType::as_name)
            .collect();
        let variants = MsgVariants::new(
            self.item.as_variants(),
            msg_ty,
            &associated_names,
            where_clause,
        );

        EnumMessage::new(
            self.item,
            msg_ty,
            &self.custom,
            variants,
            &self.associated_types,
        )
        .emit()
    }
}

impl<'a> ImplInput<'a> {
    pub fn new(attributes: &'a ContractArgs, item: &'a ItemImpl) -> Self {
        let generics = item.generics.params.iter().collect();
        let error = ContractErrorAttr::new(item);
        let custom = Custom::new(&item.attrs);
        let override_entry_points = OverrideEntryPoints::new(&item.attrs);
        let interfaces = Interfaces::new(item);

        Self {
            attributes,
            item,
            generics,
            error,
            custom,
            override_entry_points,
            interfaces,
        }
    }

    pub fn process(&self) -> TokenStream {
        match is_trait(self.item) {
            true => self.process_interface(),
            false => self.process_contract(),
        }
    }

    fn process_interface(&self) -> TokenStream {
        let multitest_helpers = self.emit_multitest_helpers();
        let querier = self.emit_querier_for_bound_impl();

        #[cfg(not(tarpaulin_include))]
        quote! {
            pub mod sv {
                use super::*;

                #multitest_helpers

                #querier
            }
        }
    }

    fn process_contract(&self) -> TokenStream {
        let Self {
            item,
            generics,
            custom,
            interfaces,
            ..
        } = self;
        let multitest_helpers = self.emit_multitest_helpers();

        let querier = ContractQuerier::new(item, interfaces).emit();
        let messages = self.emit_messages();
        let remote = ContractRemote::new(item, interfaces).emit();
        let contract_api = ContractApi::new(item, generics, custom, interfaces).emit();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod sv {
                    use super::*;

                    #messages

                    #multitest_helpers

                    #remote

                    #querier

                    #contract_api
                }
            }
        }
    }

    fn emit_messages(&self) -> TokenStream {
        let instantiate = self.emit_struct_msg(MsgType::Instantiate);
        let migrate = self.emit_struct_msg(MsgType::Migrate);
        let exec_impl = self.emit_enum_msg(MsgType::Exec);
        let query_impl = self.emit_enum_msg(MsgType::Query);
        let exec = self.emit_glue_msg(MsgType::Exec);
        let query = self.emit_glue_msg(MsgType::Query);

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
        StructMessage::new(self.item, msg_ty, &self.generics, &self.custom)
            .map_or(quote! {}, |msg| msg.emit())
    }

    fn emit_enum_msg(&self, msg_ty: MsgType) -> TokenStream {
        ContractEnumMessage::new(self.item, msg_ty, &self.generics, &self.error, &self.custom)
            .emit()
    }

    fn emit_glue_msg(&self, msg_ty: MsgType) -> TokenStream {
        let Self { generics, item, .. } = self;
        let where_clause = &item.generics.where_clause;
        let variants = MsgVariants::new(item.as_variants(), msg_ty, generics, where_clause);
        GlueMessage::new(
            self.item,
            msg_ty,
            &self.error,
            &self.custom,
            &self.interfaces,
            variants,
        )
        .emit()
    }

    fn emit_querier_for_bound_impl(&self) -> TokenStream {
        let contract_module = self.attributes.module.as_ref();
        let variants_args =
            MsgVariants::<GenericParam>::new(self.item.as_variants(), MsgType::Query, &[], &None);
        let associated_types = ImplAssociatedTypes::new(self.item);
        ImplQuerier::new(
            self.item,
            &variants_args,
            &associated_types,
            &self.interfaces,
            &contract_module,
        )
        .emit()
    }

    fn emit_multitest_helpers(&self) -> TokenStream {
        if !cfg!(feature = "mt") {
            return quote! {};
        }

        let Self {
            item,
            custom,
            override_entry_points,
            interfaces,
            ..
        } = self;
        let contract_module = self.attributes.module.as_ref();
        let generic_params = &self.generics;

        if is_trait(item) {
            ImplMtHelpers::new(item, generic_params, custom, interfaces, &contract_module).emit()
        } else {
            ContractMtHelpers::new(item, generic_params, custom, override_entry_points).emit()
        }
    }
}
