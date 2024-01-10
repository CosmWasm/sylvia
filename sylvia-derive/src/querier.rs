use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::ItemImpl;
use syn::Path;

use crate::associated_types::AssociatedTypes;
use crate::associated_types::ImplAssociatedTypes;
use crate::check_generics::GetPath;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::message::{MsgVariant, MsgVariants};
use crate::parser::MsgType;

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

        let methods_impl = variants
            .variants()
            .iter()
            .map(|variant| variant.emit_trait_querier_impl(&associated_types.as_names()));

        let methods_declaration = variants
            .variants()
            .iter()
            .map(MsgVariant::emit_querier_declaration);

        let generics = associated_types.as_names();
        let types_declaration = associated_types.as_types_declaration();
        let types_definition = associated_types.emit_types_definition();
        let where_clause = associated_types.as_where_clause();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub struct BoundQuerier<'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > {
                    contract: &'a #sylvia ::cw_std::Addr,
                    querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>,
                    _phantom: std::marker::PhantomData<( #(#generics,)* )>,
                }

                impl<'a, C: #sylvia ::cw_std::CustomQuery, #(#generics,)* > BoundQuerier<'a, C, #(#generics,)* > #where_clause {
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
}

pub struct ImplQuerier<'a, Generic> {
    source: &'a ItemImpl,
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a ImplAssociatedTypes<'a>,
    interfaces: &'a Interfaces,
    contract_module: &'a Option<&'a Path>,
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
        contract_module: &'a Option<&'a Path>,
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
        let accessor = MsgType::Query.as_accessor_name(false);

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
        let bound_querier = contract_module
            .map(|module| quote! { #module ::sv::BoundQuerier})
            .unwrap_or_else(|| quote! { sv::BoundQuerier });

        let types_definition = associated_types.as_item_types();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                impl <'a, C: #sylvia ::cw_std::CustomQuery, #generic_params > #querier for #bound_querier<'a, C, #generic_params > #where_clause {
                    #(#types_definition)*

                    #(#methods_impl)*
                }
            }
        }
    }
}
