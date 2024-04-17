use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, GenericParam, ItemImpl, ItemTrait, TraitItem, Type};

use crate::associated_types::AssociatedTypes;
use crate::crate_module;
use crate::message::{MsgVariant, MsgVariants};
use crate::parser::attributes::msg::MsgType;
use crate::parser::{
    Custom, FilteredOverrideEntryPoints, OverrideEntryPoint, ParsedSylviaAttributes,
};
use crate::utils::emit_bracketed_generics;
use crate::variant_descs::AsVariantDescs;

fn get_ident_from_type(contract_name: &Type) -> &Ident {
    let Type::Path(type_path) = contract_name else {
        unreachable!()
    };
    let segments = &type_path.path.segments;
    assert!(!segments.is_empty());
    let segment = &segments.last().unwrap();
    &segment.ident
}

pub struct ContractMtHelpers<'a> {
    error_type: Type,
    contract_name: &'a Type,
    source: &'a ItemImpl,
    generic_params: &'a [&'a GenericParam],
    where_clause: &'a Option<syn::WhereClause>,
    custom: &'a Custom,
    override_entry_points: Vec<OverrideEntryPoint>,
    instantiate_variants: MsgVariants<'a, GenericParam>,
    exec_variants: MsgVariants<'a, GenericParam>,
    query_variants: MsgVariants<'a, GenericParam>,
    migrate_variants: MsgVariants<'a, GenericParam>,
    reply_variants: MsgVariants<'a, GenericParam>,
    sudo_variants: MsgVariants<'a, GenericParam>,
}

impl<'a> ContractMtHelpers<'a> {
    pub fn new(
        source: &'a ItemImpl,
        generic_params: &'a [&'a GenericParam],
        custom: &'a Custom,
        override_entry_points: Vec<OverrideEntryPoint>,
    ) -> Self {
        let where_clause = &source.generics.where_clause;
        let instantiate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Instantiate,
            generic_params,
            where_clause,
        );
        let exec_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Exec,
            generic_params,
            where_clause,
        );
        let query_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Query,
            generic_params,
            where_clause,
        );
        let migrate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Migrate,
            generic_params,
            where_clause,
        );
        let reply_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Reply,
            generic_params,
            where_clause,
        );
        let sudo_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Sudo,
            generic_params,
            where_clause,
        );

        let error_type = ParsedSylviaAttributes::new(source.attrs.iter())
            .error_attrs
            .unwrap_or_default()
            .error;
        let error_type = parse_quote! { #error_type };

        let contract_name = &source.self_ty;

        Self {
            error_type,
            source,
            generic_params,
            where_clause,
            contract_name,
            custom,
            override_entry_points,
            instantiate_variants,
            exec_variants,
            query_variants,
            sudo_variants,
            migrate_variants,
            reply_variants,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            error_type,
            contract_name,
            custom,
            exec_variants,
            query_variants,
            migrate_variants,
            sudo_variants,
            generic_params,
            where_clause,
            ..
        } = self;
        let sylvia = crate_module();

        let custom_msg = custom.msg_or_default();
        let mt_app: Type = parse_quote! {
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
        let api = quote! { < #contract_name as #sylvia ::types::ContractApi> };

        let exec_methods =
            exec_variants.emit_mt_method_definitions(&custom_msg, &mt_app, error_type, &api);
        let query_methods =
            query_variants.emit_mt_method_definitions(&custom_msg, &mt_app, error_type, &api);
        let sudo_methods =
            sudo_variants.emit_mt_method_definitions(&custom_msg, &mt_app, error_type, &api);
        let migrate_methods =
            migrate_variants.emit_mt_method_definitions(&custom_msg, &mt_app, error_type, &api);

        let exec_methods_declarations =
            exec_variants.emit_mt_method_declarations(&custom_msg, error_type, &api);
        let query_methods_declarations =
            query_variants.emit_mt_method_declarations(&custom_msg, error_type, &api);
        let sudo_methods_declarations =
            sudo_variants.emit_mt_method_declarations(&custom_msg, error_type, &api);
        let migrate_methods_declarations =
            migrate_variants.emit_mt_method_declarations(&custom_msg, error_type, &api);

        let where_predicates = where_clause
            .as_ref()
            .map(|where_clause| &where_clause.predicates);

        let contract_block = self.generate_contract_helpers();

        let contract = get_ident_from_type(contract_name);
        let trait_name = Ident::new(&format!("{}Proxy", contract), contract.span());

        quote! {
            pub mod mt {
                use super::*;
                use #sylvia ::cw_multi_test::Executor;

                pub trait #trait_name <'app, MtApp, #(#generic_params,)* >
                    #where_clause
                {
                    #( #exec_methods_declarations )*
                    #( #migrate_methods_declarations )*
                    #( #query_methods_declarations )*
                    #( #sudo_methods_declarations )*
                }

                impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#generic_params,)* >
                    #trait_name <'app, #mt_app, #(#generic_params,)* >
                        for #sylvia ::multitest::Proxy <'app, #mt_app, #contract_name >
                    where
                        CustomT: #sylvia ::cw_multi_test::Module,
                        CustomT::ExecT: #sylvia::types::CustomMsg + 'static,
                        CustomT::QueryT: #sylvia ::types::CustomQuery + 'static,
                        WasmT: #sylvia ::cw_multi_test::Wasm<CustomT::ExecT, CustomT::QueryT>,
                        BankT: #sylvia ::cw_multi_test::Bank,
                        ApiT: #sylvia ::cw_std::Api,
                        StorageT: #sylvia ::cw_std::Storage,
                        StakingT: #sylvia ::cw_multi_test::Staking,
                        DistrT: #sylvia ::cw_multi_test::Distribution,
                        IbcT: #sylvia ::cw_multi_test::Ibc,
                        GovT: #sylvia ::cw_multi_test::Gov,
                        #mt_app : Executor< #custom_msg >,
                        #where_predicates
                {
                    #( #exec_methods )*
                    #( #migrate_methods )*
                    #( #query_methods )*
                    #( #sudo_methods )*
                }

                #contract_block
            }
        }
    }

    fn generate_contract_helpers(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            error_type,
            generic_params,
            where_clause,
            contract_name,
            instantiate_variants,
            ..
        } = self;

        let fields_names = instantiate_variants
            .get_only_variant()
            .map(MsgVariant::as_fields_names)
            .unwrap_or(vec![]);

        let fields = instantiate_variants
            .get_only_variant()
            .map(MsgVariant::emit_fields)
            .unwrap_or(vec![]);

        let used_generics = instantiate_variants.used_generics();
        let bracketed_used_generics = emit_bracketed_generics(used_generics);

        let where_predicates = where_clause
            .as_ref()
            .map(|where_clause| &where_clause.predicates);

        let contract = get_ident_from_type(contract_name);
        let contract = if !generic_params.is_empty() {
            quote! { #contract ::< #(#generic_params,)* > }
        } else {
            quote! { #contract }
        };

        let instantiate_msg = if !used_generics.is_empty() {
            quote! { InstantiateMsg::< #(#used_generics,)* > }
        } else {
            quote! { InstantiateMsg }
        };

        let impl_contract = self.generate_impl_contract();

        let custom_msg = self.custom.msg_or_default();
        let custom_query = self.custom.query_or_default();

        let mt_app = quote! {
            #sylvia ::cw_multi_test::App<
                BankT,
                ApiT,
                StorageT,
                CustomT,
                #sylvia ::cw_multi_test::WasmKeeper< #custom_msg , #custom_query >,
                StakingT,
                DistrT,
                IbcT,
                GovT,
            >
        };

        let instantiate2 = if cfg!(feature = "cosmwasm_1_2") {
            quote! {
                let msg = #sylvia ::cw_std::to_json_binary(&msg)
                    .map_err(Into::< #error_type >::into)?;
                let sender = #sylvia ::cw_std::Addr::unchecked(sender);

                let msg = #sylvia ::cw_std::WasmMsg::Instantiate2 {
                    admin,
                    code_id: code_id.code_id,
                    msg,
                    funds: funds.to_owned(),
                    label: label.to_owned(),
                    salt: salt.into(),
                };
                let app_response = (*code_id.app)
                    .app_mut()
                    .execute(sender.clone(), msg.into())
                    .map_err(|err| err.downcast::< #error_type >().unwrap())?;

                #sylvia:: cw_utils::parse_instantiate_response_data(app_response.data.unwrap().as_slice())
                    .map_err(|err| Into::into( #sylvia ::cw_std::StdError::generic_err(err.to_string())))
                    .map(|data| #sylvia ::multitest::Proxy {
                        contract_addr: #sylvia ::cw_std::Addr::unchecked(data.contract_address),
                        app: code_id.app,
                        _phantom: std::marker::PhantomData::default(),
                    })
            }
        } else {
            quote! {
                let err = #sylvia ::cw_std::StdError::generic_err(
                    "`with_salt` was called, but it requires `cosmwasm_1_2` feature enabled. Consider removing `with_salt` or adding the `cosmwasm_1_2` feature."
                );
                Err(Into::into(err))
            }
        };

        let code_info = if cfg!(feature = "cosmwasm_1_2") {
            quote! {
                pub fn code_info(&self) -> #sylvia ::cw_std::StdResult< #sylvia ::cw_std::CodeInfoResponse> {
                    self.app.querier().query_wasm_code_info(self.code_id)
                }
            }
        } else {
            quote! {}
        };

        quote! {
            #impl_contract

            pub struct CodeId<'app, #(#generic_params,)* MtApp> {
                code_id: u64,
                app: &'app #sylvia ::multitest::App<MtApp>,
                _phantom: std::marker::PhantomData<( #(#generic_params,)* )>,

            }

            impl<'app, BankT, ApiT, StorageT, CustomT, StakingT, DistrT, IbcT, GovT, #(#generic_params,)* > CodeId<'app, #(#generic_params,)* #mt_app >
                where
                    BankT: #sylvia ::cw_multi_test::Bank,
                    ApiT: #sylvia ::cw_std::Api,
                    StorageT: #sylvia ::cw_std::Storage,
                    CustomT: #sylvia ::cw_multi_test::Module<ExecT = #custom_msg, QueryT = #custom_query >,
                    StakingT: #sylvia ::cw_multi_test::Staking,
                    DistrT: #sylvia ::cw_multi_test::Distribution,
                    IbcT: #sylvia ::cw_multi_test::Ibc,
                    GovT: #sylvia ::cw_multi_test::Gov,
                    #where_predicates
            {
                pub fn store_code(app: &'app #sylvia ::multitest::App< #mt_app >) -> Self {
                    let code_id = app
                        .app_mut()
                        .store_code(Box::new(#contract ::new()));
                    Self { code_id, app, _phantom: std::marker::PhantomData::default() }
                }

                pub fn code_id(&self) -> u64 {
                    self.code_id
                }

                #code_info

                pub fn instantiate(
                    &self, #(#fields,)*
                ) -> InstantiateProxy<'_, 'app, #(#generic_params,)* #mt_app > {
                    let msg = #instantiate_msg {#(#fields_names,)*};
                    InstantiateProxy::< #(#generic_params,)* _> {
                        code_id: self,
                        funds: &[],
                        label: "Contract",
                        admin: None,
                        salt: None,
                        msg,
                    }
                }
            }

            pub struct InstantiateProxy<'proxy, 'app, #(#generic_params,)* MtApp> {
                code_id: &'proxy CodeId <'app, #(#generic_params,)* MtApp>,
                funds: &'proxy [#sylvia ::cw_std::Coin],
                label: &'proxy str,
                admin: Option<String>,
                salt: Option<&'proxy [u8]>,
                msg: InstantiateMsg #bracketed_used_generics,
            }

            impl<'proxy, 'app, #(#generic_params,)* MtApp> InstantiateProxy<'proxy, 'app, #(#generic_params,)* MtApp>
                where
                    MtApp: Executor< #custom_msg >,
                    #where_predicates
            {
                pub fn with_funds(self, funds: &'proxy [#sylvia ::cw_std::Coin]) -> Self {
                    Self { funds, ..self }
                }

                pub fn with_label(self, label: &'proxy str) -> Self {
                    Self { label, ..self }
                }

                pub fn with_admin<'s>(self, admin: impl Into<Option<&'s str>>) -> Self {
                    let admin = admin.into().map(str::to_owned);
                    Self { admin, ..self }
                }

                pub fn with_salt(self, salt: impl Into<Option<&'proxy [u8]>>) -> Self {
                    let salt = salt.into();
                    Self { salt, ..self }
                }

                #[track_caller]
                pub fn call(self, sender: &#sylvia ::cw_std::Addr ) -> Result<#sylvia ::multitest::Proxy<'app, MtApp, #contract_name >, #error_type> {
                    let Self {code_id, funds, label, admin, salt, msg} = self;

                    match salt {
                        Some(salt) => {
                            #instantiate2
                        },
                        None => (*code_id.app)
                            .app_mut()
                            .instantiate_contract(
                                code_id.code_id,
                                sender.clone(),
                                &msg,
                                funds,
                                label,
                                admin,
                            )
                            .map_err(|err| err.downcast().unwrap())
                            .map(|addr| #sylvia ::multitest::Proxy {
                                contract_addr: addr,
                                app: code_id.app,
                                _phantom: std::marker::PhantomData::default(),
                            }),
                    }
                }
            }
        }
    }

    fn generate_impl_contract(&self) -> TokenStream {
        let Self {
            source,
            contract_name,
            custom,
            override_entry_points,
            generic_params,
            migrate_variants,
            reply_variants,
            ..
        } = self;
        let sylvia = crate_module();

        let bracketed_generics = emit_bracketed_generics(generic_params);
        let full_where_clause = &source.generics.where_clause;

        let instantiate_body = override_entry_points
            .get_entry_point(MsgType::Instantiate)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Instantiate, contract_name));

        let exec_body = override_entry_points
            .get_entry_point(MsgType::Exec)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Exec, contract_name));

        let query_body = override_entry_points
            .get_entry_point(MsgType::Query)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Query, contract_name));

        let sudo_body = override_entry_points
            .get_entry_point(MsgType::Sudo)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Sudo, contract_name));

        let migrate_body = match override_entry_points.get_entry_point(MsgType::Migrate) {
            Some(entry_point) => entry_point.emit_multitest_dispatch(),
            None if migrate_variants.get_only_variant().is_some() => {
                emit_default_dispatch(&MsgType::Migrate, contract_name)
            }
            None => quote! { #sylvia ::anyhow::bail!("migrate not implemented for contract") },
        };

        let reply_body = match override_entry_points.get_entry_point(MsgType::Reply) {
            Some(entry_point) => entry_point.emit_multitest_dispatch(),
            None => reply_variants
                .get_only_variant()
                .as_ref()
                .map(|reply| {
                    let reply_name = reply.name();
                    let reply_name = Ident::new(
                        &reply_name.to_string().to_case(Case::Snake),
                        reply_name.span(),
                    );
                    quote! {
                        self. #reply_name ((deps, env).into(), msg).map_err(Into::into)
                    }
                })
                .unwrap_or_else(|| {
                    quote! {
                        #sylvia ::anyhow::bail!("reply not implemented for contract")
                    }
                }),
        };

        let custom_msg = custom.msg_or_default();
        let custom_query = custom.query_or_default();

        quote! {
            impl #bracketed_generics #sylvia ::cw_multi_test::Contract<#custom_msg, #custom_query> for #contract_name #full_where_clause {
                    fn execute(
                        &self,
                        deps: #sylvia ::cw_std::DepsMut< #custom_query >,
                        env: #sylvia ::cw_std::Env,
                        info: #sylvia ::cw_std::MessageInfo,
                        msg: Vec<u8>,
                    ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                        #exec_body
                    }

                fn instantiate(
                    &self,
                    deps: #sylvia ::cw_std::DepsMut<#custom_query>,
                    env: #sylvia ::cw_std::Env,
                    info: #sylvia ::cw_std::MessageInfo,
                    msg: Vec<u8>,
                ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                    #instantiate_body
                }

                fn query(
                    &self,
                    deps: #sylvia ::cw_std::Deps<#custom_query>,
                    env: #sylvia ::cw_std::Env,
                    msg: Vec<u8>,
                ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Binary> {
                    #query_body
                }

                fn sudo(
                    &self,
                    deps: #sylvia ::cw_std::DepsMut<#custom_query>,
                    env: #sylvia ::cw_std::Env,
                    msg: Vec<u8>,
                ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                    #sudo_body
                }

                fn reply(
                    &self,
                    deps: #sylvia ::cw_std::DepsMut<#custom_query>,
                    env: #sylvia ::cw_std::Env,
                    msg: #sylvia ::cw_std::Reply,
                ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                    #reply_body
                }

                fn migrate(
                    &self,
                    deps: #sylvia ::cw_std::DepsMut<#custom_query>,
                    env: #sylvia ::cw_std::Env,
                    msg: Vec<u8>,
                ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                    #migrate_body
                }
            }
        }
    }
}

pub struct TraitMtHelpers<'a> {
    source: &'a ItemTrait,
    error_type: Type,
    associated_types: &'a AssociatedTypes<'a>,
    exec_variants: MsgVariants<'a, GenericParam>,
    query_variants: MsgVariants<'a, GenericParam>,
    sudo_variants: MsgVariants<'a, GenericParam>,
    where_clause: &'a Option<syn::WhereClause>,
}

impl<'a> TraitMtHelpers<'a> {
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

        let associated_args_for_api: Vec<_> = associated_types
            .without_special()
            .map(|associated| {
                let assoc = &associated.ident;
                quote! { Self:: #assoc }
            })
            .collect();

        let bracketed_generics = emit_bracketed_generics(&associated_args_for_api);
        let api = quote! { < Api #bracketed_generics as #sylvia ::types::InterfaceApi> };

        let associated_types_declaration = associated_types.without_error();

        let exec_methods = exec_variants.emit_mt_method_definitions(
            &custom_msg,
            &mt_app,
            &prefixed_error_type,
            &api,
        );
        let query_methods = query_variants.emit_mt_method_definitions(
            &custom_msg,
            &mt_app,
            &prefixed_error_type,
            &api,
        );
        let sudo_methods = sudo_variants.emit_mt_method_definitions(
            &custom_msg,
            &mt_app,
            &prefixed_error_type,
            &api,
        );

        let exec_methods_declarations =
            exec_variants.emit_mt_method_declarations(&custom_msg, &prefixed_error_type, &api);
        let query_methods_declarations =
            query_variants.emit_mt_method_declarations(&custom_msg, &prefixed_error_type, &api);
        let sudo_methods_declarations =
            sudo_variants.emit_mt_method_declarations(&custom_msg, &prefixed_error_type, &api);

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

fn emit_default_dispatch(msg_ty: &MsgType, contract_name: &Type) -> TokenStream {
    let sylvia = crate_module();

    let values = msg_ty.emit_ctx_values();
    let msg_name = msg_ty.as_accessor_wrapper_name();
    let api_msg = quote! { < #contract_name as #sylvia ::types::ContractApi> :: #msg_name };

    quote! {
        #sylvia ::cw_std::from_json::< #api_msg >(&msg)?
            .dispatch(self, ( #values ))
            .map_err(Into::into)
    }
}
