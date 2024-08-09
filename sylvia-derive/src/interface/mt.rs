use convert_case::Case;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, GenericParam, ItemTrait, TraitItem, Type};

use crate::crate_module;
use crate::parser::attributes::msg::MsgType;
use crate::parser::variant_descs::AsVariantDescs;
use crate::types::associated_types::AssociatedTypes;
use crate::types::msg_variant::{MsgVariant, MsgVariants};
use crate::utils::SvCasing;

/// Emits helpers for testing interface messages using MultiTest.
pub struct MtHelpers<'a> {
    source: &'a ItemTrait,
    error_type: Type,
    associated_types: &'a AssociatedTypes<'a>,
    exec_variants: MsgVariants<'a, GenericParam>,
    query_variants: MsgVariants<'a, GenericParam>,
    sudo_variants: MsgVariants<'a, GenericParam>,
    where_clause: &'a Option<syn::WhereClause>,
}

impl<'a> MtHelpers<'a> {
    pub fn new(source: &'a ItemTrait, associated_types: &'a AssociatedTypes) -> Self {
        let where_clause = &source.generics.where_clause;
        let exec_variants =
            MsgVariants::new(source.as_variants(), MsgType::Exec, &[], where_clause);
        let query_variants =
            MsgVariants::new(source.as_variants(), MsgType::Query, &[], where_clause);
        let sudo_variants =
            MsgVariants::new(source.as_variants(), MsgType::Sudo, &[], where_clause);
        let associated_error = source.items.iter().find_map(|item| match item {
            TraitItem::Type(ty) if ty.ident == "Error" => Some(&ty.ident),
            _ => None,
        });
        let error_type: Type = match associated_error {
            Some(error) => parse_quote!(#error),
            // This should never happen as the `interface` macro requires the trait to have an associated `Error` type
            None => unreachable!(),
        };

        Self {
            error_type,
            source,
            associated_types,
            where_clause,
            exec_variants,
            query_variants,
            sudo_variants,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            error_type,
            source,
            associated_types,
            where_clause,
            exec_variants,
            query_variants,
            sudo_variants,
        } = self;

        let sylvia = crate_module();

        let interface_name = &source.ident;
        let trait_name = Ident::new(&format!("{}Proxy", interface_name), interface_name.span());

        let custom_msg: Type = parse_quote! { CustomMsgT };
        let prefixed_error_type: Type = parse_quote! { Self:: #error_type };

        let mt_app = parse_quote! {
            #sylvia ::cw_multi_test::App<
                BankT,
                ApiT,
                StorageT,
                CustomT,
                WasmT,
                StakingT,
                DistrT,
                IbcT,
                GovT,
            >
        };

        let associated_args: Vec<_> = associated_types
            .without_error()
            .map(|associated| &associated.ident)
            .collect();

        let api = quote! {
            < dyn #interface_name < Error = (), #(#associated_args = Self:: #associated_args,)* > as InterfaceMessagesApi >
        };

        let associated_types_declaration = associated_types.without_error();

        let exec_methods = exec_variants.variants().map(|variant| {
            variant.emit_mt_method_definition(&custom_msg, &mt_app, &prefixed_error_type, &api)
        });
        let query_methods = query_variants.variants().map(|variant| {
            variant.emit_mt_method_definition(&custom_msg, &mt_app, &prefixed_error_type, &api)
        });
        let sudo_methods = sudo_variants.variants().map(|variant| {
            variant.emit_mt_method_definition(&custom_msg, &mt_app, &prefixed_error_type, &api)
        });

        let exec_methods_declarations = exec_variants.variants().map(|variant| {
            variant.emit_mt_method_declaration(&custom_msg, &prefixed_error_type, &api)
        });
        let query_methods_declarations = query_variants.variants().map(|variant| {
            variant.emit_mt_method_declaration(&custom_msg, &prefixed_error_type, &api)
        });
        let sudo_methods_declarations = sudo_variants.variants().map(|variant| {
            variant.emit_mt_method_declaration(&custom_msg, &prefixed_error_type, &api)
        });

        let where_predicates = where_clause
            .as_ref()
            .map(|where_clause| &where_clause.predicates);

        quote! {
            pub mod mt {
                use super::*;

                pub trait #trait_name <MtApp, #custom_msg > #where_clause {
                    type #error_type: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static;
                    #(#associated_types_declaration)*

                    #(#query_methods_declarations)*
                    #(#exec_methods_declarations)*
                    #(#sudo_methods_declarations)*
                }

                impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #custom_msg, ContractT: super:: #interface_name > #trait_name < #mt_app, #custom_msg > for #sylvia ::multitest::Proxy<'_, #mt_app, ContractT >
                where
                    ContractT:: #error_type : std::fmt::Debug + std::fmt::Display + Send + Sync + 'static,
                    #custom_msg: #sylvia ::types::CustomMsg + 'static,
                    CustomT: #sylvia ::cw_multi_test::Module,
                    WasmT: #sylvia ::cw_multi_test::Wasm<CustomT::ExecT, CustomT::QueryT>,
                    BankT: #sylvia ::cw_multi_test::Bank,
                    ApiT: #sylvia ::cw_std::Api,
                    StorageT: #sylvia ::cw_std::Storage,
                    CustomT: #sylvia ::cw_multi_test::Module,
                    StakingT: #sylvia ::cw_multi_test::Staking,
                    DistrT: #sylvia ::cw_multi_test::Distribution,
                    IbcT: #sylvia ::cw_multi_test::Ibc,
                    GovT: #sylvia ::cw_multi_test::Gov,
                    CustomT::ExecT: #sylvia ::types::CustomMsg + 'static,
                    CustomT::QueryT: #sylvia:: types::CustomQuery + 'static,
                    #mt_app : #sylvia ::cw_multi_test::Executor< #custom_msg >,
                    #where_predicates
                {
                    type #error_type = <ContractT as super:: #interface_name>:: #error_type ;
                    #(type #associated_args = <ContractT as super:: #interface_name>:: #associated_args ;)*

                    #(#query_methods)*
                    #(#exec_methods)*
                    #(#sudo_methods)*
                }
            }
        }
    }
}

trait EmitMethods {
    fn emit_mt_method_definition(
        &self,
        custom_msg: &Type,
        mt_app: &Type,
        error_type: &Type,
        api: &TokenStream,
    ) -> TokenStream;

    fn emit_mt_method_declaration(
        &self,
        custom_msg: &Type,
        error_type: &Type,
        api: &TokenStream,
    ) -> TokenStream;
}

impl EmitMethods for MsgVariant<'_> {
    fn emit_mt_method_definition(
        &self,
        custom_msg: &Type,
        mt_app: &Type,
        error_type: &Type,
        api: &TokenStream,
    ) -> TokenStream {
        let sylvia = crate_module();

        let name = self.name();
        let return_type = self.return_type();

        let params: Vec<_> = self
            .fields()
            .iter()
            .map(|field| field.emit_method_field_folded())
            .collect();
        let arguments = self.as_fields_names();
        let type_name = self.msg_type().as_accessor_name();
        let name = name.to_case(Case::Snake);

        match self.msg_type() {
            MsgType::Exec => quote! {
                #[track_caller]
                fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::< #error_type, #api :: #type_name, #mt_app, #custom_msg> {
                    let msg = #api :: #type_name :: #name ( #(#arguments),* );

                    #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                }
            },
            MsgType::Query => {
                quote! {
                    fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                        let msg = #api :: #type_name :: #name ( #(#arguments),* );

                        (*self.app)
                            .querier()
                            .query_wasm_smart(self.contract_addr.clone(), &msg)
                            .map_err(Into::into)
                    }
                }
            }
            MsgType::Sudo => quote! {
                fn #name (&self, #(#params,)* ) -> Result< #sylvia ::cw_multi_test::AppResponse, #error_type> {
                    let msg = #api :: #type_name :: #name ( #(#arguments),* );

                    (*self.app)
                        .app_mut()
                        .wasm_sudo(self.contract_addr.clone(), &msg)
                        .map_err(|err| err.downcast().unwrap())
                }
            },
            MsgType::Migrate => quote! {
                #[track_caller]
                fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::MigrateProxy::< #error_type, #api :: #type_name , #mt_app, #custom_msg> {
                    let msg = #api :: #type_name ::new( #(#arguments),* );

                    #sylvia ::multitest::MigrateProxy::new(&self.contract_addr, msg, &self.app)
                }
            },
            _ => quote! {},
        }
    }

    fn emit_mt_method_declaration(
        &self,
        custom_msg: &Type,
        error_type: &Type,
        api: &TokenStream,
    ) -> TokenStream {
        let sylvia = crate_module();

        let name = self.name();
        let return_type = self.return_type();

        let params: Vec<_> = self
            .fields()
            .iter()
            .map(|field| field.emit_method_field_folded())
            .collect();
        let type_name = self.msg_type().as_accessor_name();
        let name = name.to_case(Case::Snake);

        match self.msg_type() {
            MsgType::Exec => quote! {
                fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::< #error_type, #api:: #type_name, MtApp, #custom_msg>;
            },
            MsgType::Query => quote! {
                fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type>;
            },
            MsgType::Sudo => quote! {
                fn #name (&self, #(#params,)* ) -> Result< #sylvia ::cw_multi_test::AppResponse, #error_type>;
            },
            MsgType::Migrate => quote! {
                #[track_caller]
                fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::MigrateProxy::< #error_type, #api :: #type_name, MtApp, #custom_msg>;
            },
            _ => quote! {},
        }
    }
}
