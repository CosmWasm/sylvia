use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericParam, Ident, ItemImpl};

use crate::associated_types::{AssociatedTypes, EmitAssociated, ItemType};
use crate::check_generics::GetPath;
use crate::crate_module;
use crate::message::MsgVariants;
use crate::parser::attributes::msg::MsgType;
use crate::variant_descs::AsVariantDescs;

pub struct TraitQuerier<'a, Generic> {
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a AssociatedTypes<'a>,
    interface_name: &'a Ident,
}

impl<'a, Generic> TraitQuerier<'a, Generic>
where
    Generic: GetPath + PartialEq + ToTokens,
{
    pub fn new(
        variants: &'a MsgVariants<'a, Generic>,
        associated_types: &'a AssociatedTypes,
        interface_name: &'a Ident,
    ) -> Self {
        Self {
            variants,
            associated_types,
            interface_name,
        }
    }

    pub fn emit_trait_querier(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            variants,
            associated_types,
            interface_name,
        } = self;

        let generics: Vec<_> = associated_types
            .without_special()
            .map(ItemType::as_name)
            .collect();

        let assoc_types: Vec<_> = associated_types
            .without_special()
            .map(ItemType::as_name)
            .map(|assoc| quote! {Self:: #assoc})
            .collect();
        let methods_trait_impl = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_trait_querier_impl(&assoc_types))
            .collect::<Vec<_>>();

        let methods_declaration = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_querier_declaration(&generics));

        let types_declaration = associated_types.filtered();
        let where_clause = associated_types.as_where_clause();

        quote! {
            pub trait Querier {
                #(#types_declaration)*

                #(#methods_declaration)*
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)*> Querier for #sylvia ::types::BoundQuerier<'a, C, std::marker::PhantomData< (#(#generics,)*) > > #where_clause {
                #(type #generics = #generics;)*

                #(#methods_trait_impl)*
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery, Contract: #interface_name> Querier for #sylvia ::types::BoundQuerier<'a, C, Contract> {
                #(type #generics = <Contract as #interface_name > :: #generics;)*

                #(#methods_trait_impl)*
            }
        }
    }
}

pub struct ContractQuerier<'a> {
    source: &'a ItemImpl,
    variants: MsgVariants<'a, GenericParam>,
}

impl<'a> ContractQuerier<'a> {
    pub fn new(source: &'a ItemImpl) -> Self {
        let variants = MsgVariants::new(source.as_variants(), MsgType::Query, &[], &None);
        Self { source, variants }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source, variants, ..
        } = self;

        let where_clause = &source.generics.where_clause;
        let generics: Vec<_> = source.generics.params.iter().collect();
        let contract = &source.self_ty;

        let accessor = MsgType::Query.as_accessor_name();
        let api_path = quote! { < #contract as #sylvia ::types::ContractApi>:: #accessor };
        let methods_impl = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_querier_impl::<GenericParam>(&api_path, &generics));

        let methods_declaration = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_querier_declaration(&generics));

        let types_declaration = where_clause
            .as_ref()
            .map(EmitAssociated::emit_declaration)
            .unwrap_or(vec![]);

        let types_implementation = where_clause
            .as_ref()
            .map(EmitAssociated::emit_implementation)
            .unwrap_or(vec![]);

        let contract_name = *source.self_ty.clone();

        quote! {
            pub trait Querier {
                #(#types_declaration)*

                #(#methods_declaration)*
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)*> Querier for #sylvia ::types::BoundQuerier<'a, C, #contract_name > #where_clause {
                #(#types_implementation)*

                #(#methods_impl)*
            }
        }
    }
}
