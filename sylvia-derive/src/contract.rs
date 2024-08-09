use communication::api::Api;
use communication::enum_msg::EnumMessage;
use communication::executor::Executor;
use communication::querier::Querier;
use communication::struct_msg::StructMessage;
use communication::wrapper_msg::GlueMessage;
use mt::MtHelpers;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, ItemImpl};

use crate::parser::attributes::msg::MsgType;
use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::{
    assert_new_method_defined, ContractErrorAttr, Custom, OverrideEntryPoint,
    ParsedSylviaAttributes,
};
use crate::types::interfaces::Interfaces;
use crate::types::msg_variant::MsgVariants;

mod communication;
mod mt;

/// Preprocessed `contract` macro input for struct impl block.
///
/// Generates:
///     - [Messages](https://cosmwasm-docs.vercel.app/sylvia/macros/generated-types/message-types#contract-messages)
///         - InstantiateMsg
///         - ExecMsg
///         - QueryMsg
///         - SudoMsg
///         - MigrateMsg
///         - ContractExecMsg
///         - ContractQueryMsg
///         - ContractSudoMsg
///     - [MultiTest](https://cosmwasm-docs.vercel.app/sylvia/macros/generated-types/multitest) helpers
///     - [Querier](https://cosmwasm-docs.vercel.app/cw-multi-test) trait implementation
///     - [Executor](https://cosmwasm-docs.vercel.app/cw-multi-test) trait implementation
///     - Api trait implementation
pub struct ContractInput<'a> {
    item: &'a ItemImpl,
    generics: Vec<&'a GenericParam>,
    error: ContractErrorAttr,
    custom: Custom,
    override_entry_points: Vec<OverrideEntryPoint>,
    interfaces: Interfaces,
}

impl<'a> ContractInput<'a> {
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

    /// Process the input and generate the contract code.
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
        let executor = Executor::new(
            item.generics.clone(),
            *item.self_ty.clone(),
            executor_variants,
        )
        .emit();
        let querier = Querier::new(
            item.generics.clone(),
            *item.self_ty.clone(),
            querier_variants,
        )
        .emit();
        let messages = self.emit_messages();
        let contract_api = Api::new(item, generics, custom).emit();

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
        StructMessage::new(self.item, msg_ty, &self.generics, &self.error, &self.custom)
            .map_or(quote! {}, |msg| msg.emit())
    }

    fn emit_enum_msg(&self, msg_ty: MsgType) -> TokenStream {
        EnumMessage::new(self.item, msg_ty, &self.generics, &self.error, &self.custom).emit()
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
        MtHelpers::new(item, generic_params, custom, override_entry_points.clone()).emit()
    }
}
