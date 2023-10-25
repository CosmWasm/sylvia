use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{
    parse_quote, GenericArgument, GenericParam, Ident, ItemImpl, ItemTrait, PathArguments,
    TraitItem, Type,
};

use crate::check_generics::GetPath;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::message::{
    ContractEnumMessage, EnumMessage, GlueMessage, InterfaceMessages, MsgVariants, StructMessage,
};
use crate::multitest::{MultitestHelpers, TraitMultitestHelpers};
use crate::parser::{ContractArgs, ContractErrorAttr, Custom, MsgType, OverrideEntryPoints};
use crate::remote::Remote;
use crate::variant_descs::AsVariantDescs;

/// Preprocessed `interface` macro input
pub struct TraitInput<'a> {
    item: &'a ItemTrait,
    generics: Vec<&'a GenericParam>,
    custom: Custom<'a>,
}

/// Preprocessed `contract` macro input for non-trait impl block
pub struct ImplInput<'a> {
    attributes: &'a ContractArgs,
    error: Type,
    item: &'a ItemImpl,
    generics: Vec<&'a GenericParam>,
    custom: Custom<'a>,
    override_entry_points: OverrideEntryPoints,
    interfaces: Interfaces,
}

impl<'a> TraitInput<'a> {
    #[cfg(not(tarpaulin_include))]
    // This requires invalid implementation which would fail at compile time and making it impossible to test
    pub fn new(item: &'a ItemTrait) -> Self {
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

        let custom = Custom::new(&item.attrs);

        Self {
            item,
            generics,
            custom,
        }
    }

    pub fn process(&self) -> TokenStream {
        let messages = self.emit_messages();
        let multitest_helpers = self.emit_helpers();
        let remote = Remote::new(&Interfaces::default()).emit();

        let querier = MsgVariants::new(
            self.item.as_variants(),
            MsgType::Query,
            &self.generics,
            &self.item.generics.where_clause,
        )
        .emit_querier();

        let interface_messages = InterfaceMessages::new(self.item, &self.generics).emit();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod sv {
                    use super::*;
                    #messages

                    #multitest_helpers

                    #remote

                    #querier

                    #interface_messages
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
        let exec = self.emit_msg(&Ident::new("ExecMsg", Span::mixed_site()), MsgType::Exec);
        let query = self.emit_msg(&Ident::new("QueryMsg", Span::mixed_site()), MsgType::Query);

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #exec

                #query
            }
        }
    }

    fn emit_msg(&self, name: &Ident, msg_ty: MsgType) -> TokenStream {
        EnumMessage::new(name, self.item, msg_ty, &self.generics, &self.custom).emit()
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
        let is_trait = self.item.trait_.is_some();

        match is_trait {
            true => self.process_interface(),
            false => self.process_contract(),
        }
    }

    fn process_interface(&self) -> TokenStream {
        let interface_generics = self.extract_generic_argument();
        let multitest_helpers = self.emit_multitest_helpers(&interface_generics);
        let querier_bound_for_impl = self.emit_querier_for_bound_impl();

        #[cfg(not(tarpaulin_include))]
        quote! {
            pub mod sv {
                use super::*;

                #multitest_helpers

                #querier_bound_for_impl
            }
        }
    }

    fn process_contract(&self) -> TokenStream {
        let Self { item, generics, .. } = self;
        let multitest_helpers = self.emit_multitest_helpers(generics);
        let where_clause = &item.generics.where_clause;

        let querier = MsgVariants::new(
            self.item.as_variants(),
            MsgType::Query,
            generics,
            where_clause,
        )
        .emit_querier();
        let messages = self.emit_messages();
        let remote = Remote::new(&self.interfaces).emit();
        let querier_from_impl = self.interfaces.emit_querier_from_impl();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #messages

                #multitest_helpers

                #remote

                #querier

                #(#querier_from_impl)*
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

    /// This method should only be called for trait impl block
    fn extract_generic_argument(&self) -> Vec<&GenericArgument> {
        let interface_generics = &self.item.trait_.as_ref();
        let args = match interface_generics {
            Some((_, path, _)) => path.segments.last().map(|segment| &segment.arguments),
            None => None,
        };

        match args {
            Some(PathArguments::AngleBracketed(args)) => {
                args.args.pairs().map(|pair| *pair.value()).collect()
            }
            _ => vec![],
        }
    }

    fn emit_querier_for_bound_impl(&self) -> TokenStream {
        let trait_module = self
            .interfaces
            .get_only_interface()
            .map(|interface| &interface.module);
        let contract_module = self.attributes.module.as_ref();
        let generics = self.extract_generic_argument();

        let variants = MsgVariants::new(self.item.as_variants(), MsgType::Query, &generics, &None);

        variants.emit_querier_for_bound_impl(trait_module, contract_module)
    }

    fn emit_multitest_helpers<Generic>(&self, generics: &[&Generic]) -> TokenStream
    where
        Generic: ToTokens + PartialEq + GetPath,
    {
        let Self {
            item,
            error,
            custom,
            override_entry_points,
            interfaces,
            ..
        } = self;

        let is_trait = self.item.trait_.is_some();

        if cfg!(feature = "mt") {
            MultitestHelpers::new(
                item,
                is_trait,
                error,
                generics,
                custom,
                override_entry_points,
                interfaces,
            )
            .emit()
        } else {
            quote! {}
        }
    }
}
