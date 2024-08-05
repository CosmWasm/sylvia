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
use crate::utils::{emit_bracketed_generics, SvCasing};

pub struct Executor<'a, Generic> {
    variants: &'a MsgVariants<'a, Generic>,
    associated_types: &'a AssociatedTypes<'a>,
    interface_name: &'a Ident,
}

impl<'a, Generic> Executor<'a, Generic>
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

    pub fn emit_executor_trait(&self) -> TokenStream {
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

        let accessor = MsgType::Exec.as_accessor_name();
        let executor_api_path =
            quote! { < Api #bracketed_generics as #sylvia ::types::InterfaceApi>:: #accessor };

        let methods_trait_impl = variants
            .variants()
            .map(|variant| variant.emit_executor_impl(&executor_api_path))
            .collect::<Vec<_>>();

        let executor_methods_declaration = variants
            .variants()
            .map(|variant| variant.emit_executor_method_declaration());

        let types_declaration = associated_types.filtered().collect::<Vec<_>>();
        let where_clause = associated_types.as_where_clause();

        quote! {
            pub trait Executor {
                #(#types_declaration)*
                #(#executor_methods_declaration)*
            }

            impl <#(#all_generics,)*> Executor
                for #sylvia ::types::ExecutorBuilder<(#sylvia ::types::EmptyExecutorBuilderState, dyn #interface_name <#( #all_generics = #all_generics,)* > ) > #where_clause {
                #(type #generics = #generics;)*
                #(#methods_trait_impl)*
            }

            impl <Contract: #interface_name> Executor
                for #sylvia ::types::ExecutorBuilder<( #sylvia ::types::EmptyExecutorBuilderState, Contract )> {
                #(type #generics = <Contract as #interface_name > :: #generics;)*
                #(#methods_trait_impl)*
            }
        }
    }
}

trait EmitExecutorMethod {
    fn emit_executor_impl(&self, api_path: &TokenStream) -> TokenStream;
    fn emit_executor_method_declaration(&self) -> TokenStream;
}

impl EmitExecutorMethod for MsgVariant<'_> {
    fn emit_executor_impl(&self, api_path: &TokenStream) -> TokenStream {
        let name = self.name();
        let fields = self.fields();
        let sylvia = crate_module();

        let parameters = fields.iter().map(MsgField::emit_method_field_folded);
        let fields_names = fields.iter().map(MsgField::name);
        let variant_name = name.to_case(Case::Snake);

        quote! {
            fn #variant_name(self, #(#parameters),*) -> Result<#sylvia ::types::ExecutorBuilder< #sylvia ::types::ReadyExecutorBuilderState >, #sylvia ::cw_std::StdError> {
                Ok(#sylvia ::types::ExecutorBuilder::<#sylvia ::types::ReadyExecutorBuilderState>::new(
                    self.contract().to_owned(),
                    self.funds().to_owned(),
                    #sylvia ::cw_std::to_json_binary( & #api_path :: #variant_name (#(#fields_names),*) )?,
                ))
            }
        }
    }

    fn emit_executor_method_declaration(&self) -> TokenStream {
        let name = self.name();
        let sylvia = crate_module();

        let parameters = self
            .fields()
            .iter()
            .map(|field| field.emit_method_field_folded());
        let variant_name = name.to_case(Case::Snake);

        quote! {
            fn #variant_name(self, #(#parameters),*) -> Result< #sylvia ::types::ExecutorBuilder<#sylvia ::types::ReadyExecutorBuilderState>, #sylvia ::cw_std::StdError>;
        }
    }
}
