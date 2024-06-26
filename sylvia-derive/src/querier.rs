use convert_case::Case;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericParam, Generics, Ident, Type};

use crate::associated_types::{AssociatedTypes, EmitAssociated, ItemType};
use crate::check_generics::GetPath;
use crate::crate_module;
use crate::message::{MsgField, MsgVariant, MsgVariants};
use crate::parser::attributes::msg::MsgType;
use crate::utils::{emit_bracketed_generics, SvCasing};

pub struct InterfaceQuerier<'a, Generic> {
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a AssociatedTypes<'a>,
    interface_name: &'a Ident,
}

impl<'a, Generic> InterfaceQuerier<'a, Generic>
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

    pub fn emit_querier_trait(&self) -> TokenStream {
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
        let all_generics: Vec<_> = associated_types.all_names().collect();
        let assoc_types: Vec<_> = generics
            .iter()
            .map(|assoc| quote! {Self:: #assoc})
            .collect();
        let bracketed_generics = emit_bracketed_generics(&assoc_types);
        let accessor = MsgType::Query.as_accessor_name();
        let api_path =
            quote! { < Api #bracketed_generics as #sylvia ::types::InterfaceApi>:: #accessor };

        let methods_trait_impl = variants
            .variants()
            .map(|variant| variant.emit_querier_impl(&api_path))
            .collect::<Vec<_>>();

        let querier_methods_declaration = variants
            .variants()
            .map(|variant| variant.emit_querier_method_declaration());

        let types_declaration = associated_types.filtered().collect::<Vec<_>>();
        let where_clause = associated_types.as_where_clause();

        quote! {
            pub trait Querier {
                #(#types_declaration)*
                #(#querier_methods_declaration)*
            }

            impl <'a, C: #sylvia ::cw_std::CustomQuery, #(#all_generics,)*> Querier for #sylvia ::types::BoundQuerier<'a, C, dyn #interface_name <#( #all_generics = #all_generics,)*> > #where_clause {
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
    generics: Generics,
    self_ty: Type,
    variants: MsgVariants<'a, GenericParam>,
}

impl<'a> ContractQuerier<'a> {
    pub fn new(generics: Generics, self_ty: Type, variants: MsgVariants<'a, GenericParam>) -> Self {
        Self {
            generics,
            self_ty,
            variants,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            generics,
            self_ty,
            variants,
            ..
        } = self;

        let where_clause = &generics.where_clause;
        let generics: Vec<_> = generics.params.iter().collect();
        let contract = &self_ty;

        let accessor = MsgType::Query.as_accessor_name();
        let api_path = quote! { < #contract as #sylvia ::types::ContractApi>:: #accessor };

        let querier_methods_impl = variants
            .variants()
            .map(|variant| variant.emit_querier_impl(&api_path));

        let querier_methods_declaration = variants
            .variants()
            .map(|variant| variant.emit_querier_method_declaration());

        let types_declaration = where_clause
            .as_ref()
            .map(EmitAssociated::emit_declaration)
            .unwrap_or(vec![]);

        let types_implementation = where_clause
            .as_ref()
            .map(EmitAssociated::emit_implementation)
            .unwrap_or(vec![]);

        let bracketed_generics = if !generics.is_empty() {
            quote! { < #(#generics,)* > }
        } else {
            quote! {}
        };

        quote! {
            pub trait Querier #bracketed_generics {
                #(#types_declaration)*
                #(#querier_methods_declaration)*
            }

            impl <'sv_querier_lifetime, #(#generics,)* C: #sylvia ::cw_std::CustomQuery> Querier #bracketed_generics for #sylvia ::types::BoundQuerier<'sv_querier_lifetime, C, #contract > #where_clause {
                #(#types_implementation)*
                #(#querier_methods_impl)*
            }
        }
    }
}

trait EmitQuerierMethod {
    fn emit_querier_impl(&self, api_path: &TokenStream) -> TokenStream;
    fn emit_querier_method_declaration(&self) -> TokenStream;
}

impl EmitQuerierMethod for MsgVariant<'_> {
    fn emit_querier_impl(&self, api_path: &TokenStream) -> TokenStream {
        let sylvia = crate_module();
        let name = self.name();
        let fields = self.fields();
        let return_type = self.return_type();

        let parameters = fields.iter().map(MsgField::emit_method_field_folded);
        let fields_names = fields.iter().map(MsgField::name);
        let variant_name = name.to_case(Case::Snake);

        quote! {
            fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError> {
                let query = #api_path :: #variant_name (#(#fields_names),*);
                self.querier().query_wasm_smart(self.contract(), &query)
            }
        }
    }

    fn emit_querier_method_declaration(&self) -> TokenStream {
        let sylvia = crate_module();
        let name = self.name();
        let return_type = self.return_type();

        let parameters = self
            .fields()
            .iter()
            .map(|field| field.emit_method_field_folded());
        let variant_name = name.to_case(Case::Snake);

        quote! {
            fn #variant_name(&self, #(#parameters),*) -> Result< #return_type, #sylvia:: cw_std::StdError>;
        }
    }
}
