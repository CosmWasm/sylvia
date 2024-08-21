use communication::api::Api;
use communication::enum_msg::EnumMessage;
use communication::executor::Executor;
use communication::querier::Querier;
use mt::MtHelpers;
use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::quote;
use syn::{Ident, ItemTrait, TraitItem};

use crate::parser::attributes::msg::MsgType;
use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::{Custom, ParsedSylviaAttributes};
use crate::types::associated_types::{AssociatedTypes, ItemType, EXEC_TYPE, QUERY_TYPE};
use crate::types::msg_variant::MsgVariants;

mod communication;
mod mt;

/// Preprocessed [`interface`](crate::interface) macro input.
///
/// Generates `sv` module containing:
///     - [Messages](https://cosmwasm-docs.vercel.app/sylvia/macros/generated-types/message-types#interface-messages)
///         - ExecMsg
///         - QueryMsg
///         - SudoMsg
///     - [MultiTest](https://cosmwasm-docs.vercel.app/sylvia/macros/generated-types/multitest#proxy-trait) helpers
///     - [Querier](https://cosmwasm-docs.vercel.app/cw-multi-test) trait implementation
///     - [Executor](https://cosmwasm-docs.vercel.app/cw-multi-test) trait implementation
///     - Api trait implementation
pub struct InterfaceInput<'a> {
    item: &'a ItemTrait,
    custom: Custom,
    associated_types: AssociatedTypes<'a>,
}

impl<'a> InterfaceInput<'a> {
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
                .as_names()
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
                .as_names()
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

    /// Processes the input and generates the interface code.
    pub fn process(&self) -> TokenStream {
        let Self {
            associated_types,
            item,
            custom,
        } = self;
        let messages = self.emit_messages();
        let associated_names: Vec<_> = associated_types
            .without_error()
            .map(ItemType::as_name)
            .collect();

        let executor_variants =
            MsgVariants::new(item.as_variants(), MsgType::Exec, &associated_names, &None);
        let query_variants =
            MsgVariants::new(item.as_variants(), MsgType::Query, &associated_names, &None);
        let executor =
            Executor::new(&executor_variants, associated_types, &item.ident).emit_executor_trait();
        let querier =
            Querier::new(&query_variants, associated_types, &item.ident).emit_querier_trait();

        let interface_messages = Api::new(item, custom, associated_types).emit();

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

        let instantiate = MsgVariants::new(
            self.item.as_variants(),
            MsgType::Instantiate,
            &[] as &[&Ident],
            &None,
        );

        if let Some(msg_variant) = instantiate.variants().next() {
            emit_error!(
                msg_variant.name().span(), "The message attribute `instantiate` is not supported in interfaces.";
                note = "Contracts need to implement `instantiate` method within their `impl` block.";
            );
        }

        let migrate = MsgVariants::new(
            self.item.as_variants(),
            MsgType::Migrate,
            &[] as &[&Ident],
            &None,
        );

        if let Some(msg_variant) = migrate.variants().next() {
            emit_error!(
                msg_variant.name().span(), "The message attribute `migrate` is not supported in interfaces";
                note = "Contracts need to implement `migrate` method within their `impl` block.";
            );
        }

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

        MtHelpers::new(item, associated_types).emit()
    }
}
