use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericParam, ItemImpl, Path};

use crate::associated_types::{AssociatedTypes, EmitAssociated, ImplAssociatedTypes, ItemType};
use crate::check_generics::GetPath;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::message::MsgVariants;
use crate::parser::attributes::msg::MsgType;
use crate::variant_descs::AsVariantDescs;

pub struct TraitQuerier<'a, Generic> {
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a AssociatedTypes<'a>,
}

impl<'a, Generic> TraitQuerier<'a, Generic>
where
    Generic: GetPath + PartialEq + ToTokens,
{
    pub fn new(
        variants: &'a MsgVariants<'a, Generic>,
        associated_types: &'a AssociatedTypes,
    ) -> Self {
        Self {
            variants,
            associated_types,
        }
    }

    pub fn emit_trait_querier(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            variants,
            associated_types,
            ..
        } = self;

        let generics: Vec<_> = associated_types
            .without_special()
            .map(ItemType::as_name)
            .collect();
        let methods_impl = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_trait_querier_impl(&generics));

        let methods_declaration = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_querier_declaration(&generics));

        let types_declaration = associated_types.filtered();
        let types_definition = associated_types.emit_types_definition();
        let where_clause = associated_types.as_where_clause();

        quote! {
            pub struct BoundQuerier<'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > {
                contract: &'a #sylvia ::cw_std::Addr,
                querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>,
                _phantom: std::marker::PhantomData<( #(#generics,)* )>,
            }

            impl<'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > BoundQuerier<'a, C, #(#generics,)* > {
                pub fn querier(&self) -> &'a #sylvia ::cw_std::QuerierWrapper<'a, C> {
                    self.querier
                }

                pub fn contract(&self) -> &'a #sylvia ::cw_std::Addr {
                    self.contract
                }

                pub fn borrowed(contract: &'a #sylvia ::cw_std::Addr, querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>) -> Self {
                    Self { contract, querier, _phantom: std::marker::PhantomData }
                }
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > Querier for BoundQuerier<'a, C, #(#generics,)* > #where_clause {
                #(#types_definition)*

                #(#methods_impl)*
            }

            pub trait Querier {
                #(#types_declaration)*

                #(#methods_declaration)*
            }
        }
    }
}

pub struct ImplQuerier<'a, Generic> {
    source: &'a ItemImpl,
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a ImplAssociatedTypes<'a>,
    interfaces: &'a Interfaces,
    contract_module: &'a Path,
}

impl<'a, Generic> ImplQuerier<'a, Generic>
where
    Generic: GetPath + PartialEq + ToTokens,
{
    pub fn new(
        source: &'a ItemImpl,
        variants: &'a MsgVariants<'a, Generic>,
        associated_types: &'a ImplAssociatedTypes,
        interfaces: &'a Interfaces,
        contract_module: &'a Path,
    ) -> Self {
        Self {
            source,
            variants,
            associated_types,
            interfaces,
            contract_module,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            variants,
            associated_types,
            interfaces,
            contract_module,
        } = self;

        let trait_module = interfaces
            .get_only_interface()
            .map(|interface| &interface.module);
        let accessor = MsgType::Query.as_accessor_name();

        let generic_params = &source.generics.params;
        let where_clause = &source.generics.where_clause;
        let associated_generics = associated_types.as_types();
        let associated_names = associated_types.as_names();
        let bracketed_generics = if !associated_generics.is_empty() {
            quote! { ::< #(#associated_generics,)* > }
        } else {
            quote! {}
        };
        let api_path = quote! { < #trait_module ::sv::Api #bracketed_generics as #sylvia ::types::InterfaceApi>:: #accessor };

        let methods_impl = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_querier_impl(&api_path, &associated_names));

        let querier = trait_module
            .map(|module| quote! { #module ::sv::Querier })
            .unwrap_or_else(|| quote! { sv::Querier });

        let types_definition = associated_types.as_item_types();

        quote! {
            impl <'a, C: #sylvia ::cw_std::CustomQuery, #generic_params > #querier for #contract_module ::sv::BoundQuerier<'a, C, #generic_params > #where_clause {
                #(#types_definition)*

                #(#methods_impl)*
            }
        }
    }
}

pub struct ContractQuerier<'a> {
    source: &'a ItemImpl,
    variants: MsgVariants<'a, GenericParam>,
    interfaces: &'a Interfaces,
}

impl<'a> ContractQuerier<'a> {
    pub fn new(source: &'a ItemImpl, interfaces: &'a Interfaces) -> Self {
        let variants = MsgVariants::new(source.as_variants(), MsgType::Query, &[], &None);
        Self {
            source,
            variants,
            interfaces,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            variants,
            interfaces,
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
        let from_implementations = interfaces.emit_querier_from_impl(&generics);

        quote! {
            pub struct BoundQuerier<'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > {
                contract: &'a #sylvia ::cw_std::Addr,
                querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>,
                _phantom: std::marker::PhantomData<( #(#generics,)* )>,
            }

            impl<'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > BoundQuerier<'a, C, #(#generics,)* > {
                pub fn querier(&self) -> &'a #sylvia ::cw_std::QuerierWrapper<'a, C> {
                    self.querier
                }

                pub fn contract(&self) -> &'a #sylvia ::cw_std::Addr {
                    self.contract
                }

                pub fn borrowed(contract: &'a #sylvia ::cw_std::Addr, querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>) -> Self {
                    Self {contract, querier, _phantom: std::marker::PhantomData}
                }
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > Querier for BoundQuerier<'a, C, #(#generics,)* > #where_clause {
                #(#types_implementation)*

                #(#methods_impl)*
            }

            pub trait Querier {
                #(#types_declaration)*

                #(#methods_declaration)*
            }

            #(#from_implementations)*
        }
    }
}
