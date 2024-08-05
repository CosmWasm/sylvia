use convert_case::Case;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, Generics, Type};

use crate::crate_module;
use crate::parser::attributes::msg::MsgType;
use crate::types::associated_types::EmitAssociated;
use crate::types::msg_field::MsgField;
use crate::types::msg_variant::{MsgVariant, MsgVariants};
use crate::utils::SvCasing;

pub struct Executor<'a> {
    generics: Generics,
    self_ty: Type,
    variants: MsgVariants<'a, GenericParam>,
}

impl<'a> Executor<'a> {
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

        let accessor = MsgType::Exec.as_accessor_name();
        let executor_api_path = quote! { < #contract as #sylvia ::types::ContractApi>:: #accessor };

        let executor_methods_impl = variants
            .variants()
            .map(|variant| variant.emit_executor_impl(&executor_api_path));

        let executor_methods_declaration = variants
            .variants()
            .map(|variant| variant.emit_executor_method_declaration());

        let types_declaration = where_clause
            .as_ref()
            .map(EmitAssociated::emit_declaration)
            .unwrap_or(vec![]);

        let types_implementation = where_clause
            .as_ref()
            .map(EmitAssociated::emit_implementation)
            .unwrap_or(vec![]);

        quote! {
            pub trait Executor<#(#generics,)*> #where_clause {
                #(#types_declaration)*
                #(#executor_methods_declaration)*
            }

            impl <#(#generics,)*> Executor<#(#generics,)*>
                for #sylvia ::types::ExecutorBuilder<( #sylvia ::types::EmptyExecutorBuilderState, #contract )> #where_clause {
                #(#types_implementation)*
                #(#executor_methods_impl)*
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
