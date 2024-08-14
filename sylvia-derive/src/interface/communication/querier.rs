use convert_case::Case;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Ident;

use crate::crate_module;
use crate::parser::attributes::msg::MsgType;
use crate::parser::check_generics::GetPath;
use crate::types::associated_types::{AssociatedTypes, ItemType};
use crate::types::msg_field::MsgField;
use crate::types::msg_variant::{MsgVariant, MsgVariants};
use crate::utils::SvCasing;

/// Emits [query helper](https://cosmwasm-docs.vercel.app/sylvia/macros/generated-types/communication#query-helpers).
///
/// Generates trait containing methods for each query message variant and implements it on
/// `sylvia::types::BoundQuerier<Contract>`.
pub struct Querier<'a, Generic> {
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a AssociatedTypes<'a>,
    interface_name: &'a Ident,
}

impl<'a, Generic> Querier<'a, Generic>
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
            .without_error()
            .map(ItemType::as_name)
            .collect();
        let all_generics: Vec<_> = associated_types.as_names().collect();
        let accessor = MsgType::Query.as_accessor_name();
        let api_path = quote! {
            < dyn #interface_name < Error = (), #(#generics = Self:: #generics,)* > as InterfaceMessagesApi > :: #accessor
        };

        let methods_trait_impl = variants
            .variants()
            .map(|variant| variant.emit_querier_impl(&api_path))
            .collect::<Vec<_>>();

        let querier_methods_declaration = variants
            .variants()
            .map(|variant| variant.emit_querier_method_declaration());

        let types_declaration = associated_types.without_error().collect::<Vec<_>>();
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
