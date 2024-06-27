use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use syn::{GenericParam, Ident, ItemImpl, ItemTrait, TraitItem};

use crate::associated_types::{AssociatedTypes, ItemType, EXEC_TYPE, QUERY_TYPE};
use crate::executor::{ContractExecutor, InterfaceExecutor};
use crate::interfaces::Interfaces;
use crate::message::{
    ContractApi, ContractEnumMessage, EnumMessage, GlueMessage, InterfaceApi, MsgVariants,
    StructMessage,
};
use crate::multitest::{ContractMtHelpers, TraitMtHelpers};
use crate::parser::attributes::msg::MsgType;
use crate::parser::{
    assert_new_method_defined, ContractErrorAttr, Custom, OverrideEntryPoint,
    ParsedSylviaAttributes,
};
use crate::querier::{ContractQuerier, InterfaceQuerier};
use crate::variant_descs::AsVariantDescs;

/// Preprocessed `interface` macro input
pub struct TraitInput<'a> {
    item: &'a ItemTrait,
    custom: Custom,
    associated_types: AssociatedTypes<'a>,
}

impl<'a> TraitInput<'a> {
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

        let custom = ParsedSylviaAttributes::new(item.attrs.iter())
            .custom_attr
            .unwrap_or_default();
        let associated_types = AssociatedTypes::new(item);

        if custom.msg.is_none()
            && !associated_types
                .all_names()
                .any(|assoc_type| assoc_type == EXEC_TYPE)
        {
            emit_warning!(
                item.ident.span(), "Missing both `{}` type and `#[sv::custom(msg=...)]` defined for the trait.", EXEC_TYPE;
                note = "Implicitly it means that the trait could not be implemented for contracts that use CustomMsg different than `cosmwasm_std::Empty`";
                note = "If this behaviour is intended, please add `#[sv::custom(msg=sylvia::cw_std::Empty]` attribute.";
            );
        }

        if custom.query.is_none()
            && !associated_types
                .all_names()
                .any(|assoc_type| assoc_type == QUERY_TYPE)
        {
            emit_warning!(
                item.ident.span(), "Missing both `{}` type and `#[sv::custom(query=...)]` defined for the trait.", QUERY_TYPE;
                note = "Implicitly it means that the trait could not be implemented for contracts that use CustomQuery different than `cosmwasm_std::Empty`";
                note = "If this behaviour is intended, please add `#[sv::custom(query=sylvia::cw_std::Empty]` attribute.";
            );
        }

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
        let associated_names: Vec<_> = associated_types.as_filtered_names().collect();

        let executor_variants =
            MsgVariants::new(item.as_variants(), MsgType::Exec, &associated_names, &None);
        let query_variants =
            MsgVariants::new(item.as_variants(), MsgType::Query, &associated_names, &None);
        let executor = InterfaceExecutor::new(&executor_variants, associated_types, &item.ident)
            .emit_executor_trait();
        let querier = InterfaceQuerier::new(&query_variants, associated_types, &item.ident)
            .emit_querier_trait();

        let interface_messages = InterfaceApi::new(item, associated_types, custom).emit();

        let multitest_helpers = self.emit_multitest_helpers();

        quote! {
            pub mod sv {
                use super::*;
                #messages

                #querier

                #executor

                #interface_messages

                #multitest_helpers
            }
        }
    }

    fn emit_messages(&self) -> TokenStream {
        let exec = self.emit_msg(MsgType::Exec);
        let query = self.emit_msg(MsgType::Query);
        let sudo = self.emit_msg(MsgType::Sudo);

        quote! {
            #exec

            #query

            #sudo
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

    fn emit_multitest_helpers(&self) -> TokenStream {
        if !cfg!(feature = "mt") {
            return quote! {};
        }

        let Self { item, .. } = self;
        let associated_types = &self.associated_types;

        TraitMtHelpers::new(item, associated_types).emit()
    }
}

/// Preprocessed `contract` macro input for non-trait impl block
pub struct ImplInput<'a> {
    error: ContractErrorAttr,
    item: &'a ItemImpl,
    generics: Vec<&'a GenericParam>,
    custom: Custom,
    override_entry_points: Vec<OverrideEntryPoint>,
    interfaces: Interfaces,
}

impl<'a> ImplInput<'a> {
    pub fn new(item: &'a ItemImpl) -> Self {
        assert_new_method_defined(item);

        let generics = item.generics.params.iter().collect();
        let parsed_attrs = ParsedSylviaAttributes::new(item.attrs.iter());
        let error = parsed_attrs.error_attrs.unwrap_or_default();
        let custom = parsed_attrs.custom_attr.unwrap_or_default();
        let override_entry_points = parsed_attrs.override_entry_point_attrs;
        let interfaces = Interfaces::new(item);

        Self {
            item,
            generics,
            error,
            custom,
            override_entry_points,
            interfaces,
        }
    }

    pub fn process(&self) -> TokenStream {
        let Self {
            item,
            generics,
            custom,
            ..
        } = self;
        let multitest_helpers = self.emit_multitest_helpers();

        let executor_variants = MsgVariants::new(item.as_variants(), MsgType::Exec, &[], &None);
        let querier_variants = MsgVariants::new(item.as_variants(), MsgType::Query, &[], &None);
        let executor = ContractExecutor::new(
            item.generics.clone(),
            *item.self_ty.clone(),
            executor_variants,
        )
        .emit();
        let querier = ContractQuerier::new(
            item.generics.clone(),
            *item.self_ty.clone(),
            querier_variants,
        )
        .emit();
        let messages = self.emit_messages();
        let contract_api = ContractApi::new(item, generics, custom).emit();

        quote! {
            pub mod sv {
                use super::*;

                #messages

                #multitest_helpers

                #querier

                #executor

                #contract_api
            }
        }
    }

    fn emit_messages(&self) -> TokenStream {
        let instantiate = self.emit_struct_msg(MsgType::Instantiate);
        let migrate = self.emit_struct_msg(MsgType::Migrate);
        let exec_impl = self.emit_enum_msg(MsgType::Exec);
        let query_impl = self.emit_enum_msg(MsgType::Query);
        let sudo_impl = self.emit_enum_msg(MsgType::Sudo);
        let exec = self.emit_glue_msg(MsgType::Exec);
        let query = self.emit_glue_msg(MsgType::Query);
        let sudo = self.emit_glue_msg(MsgType::Sudo);

        quote! {
            #instantiate

            #exec_impl

            #query_impl

            #sudo_impl

            #migrate

            #exec

            #query

            #sudo
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
        GlueMessage::new(
            self.item,
            msg_ty,
            &self.error,
            &self.custom,
            &self.interfaces,
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
            ..
        } = self;

        let generic_params = &self.generics;
        ContractMtHelpers::new(item, generic_params, custom, override_entry_points.clone()).emit()
    }
}
