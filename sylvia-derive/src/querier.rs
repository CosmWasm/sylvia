use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use crate::associated_types::AssociatedTypes;
use crate::check_generics::GetPath;
use crate::crate_module;
use crate::message::{MsgVariant, MsgVariants};
use crate::parser::MsgType;

pub struct Querier<'a, Generic> {
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a AssociatedTypes<'a>,
}

impl<'a, Generic> Querier<'a, Generic>
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
            .filter(|variant| variant.is_of_type(MsgType::Query))
            .map(|variant| variant.emit_trait_querier_impl(&associated_types.as_names()));

        let methods_declaration = variants
            .variants()
            .iter()
            .filter(|variant| variant.is_of_type(MsgType::Query))
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
