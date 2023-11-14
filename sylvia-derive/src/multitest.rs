use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::visit::Visit;
use syn::{parse_quote, GenericArgument, GenericParam, ImplItem, ItemImpl, ItemTrait, Path, Type};

use crate::check_generics::CheckGenerics;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::message::{MsgVariant, MsgVariants};
use crate::parser::{ContractErrorAttr, Custom, MsgType, OverrideEntryPoint, OverrideEntryPoints};
use crate::utils::{emit_bracketed_generics, filter_wheres, is_trait};
use crate::variant_descs::AsVariantDescs;

fn interface_name(source: &ItemImpl) -> &Ident {
    let trait_name = &source.trait_;
    let Some(trait_name) = trait_name else {
        unreachable!()
    };
    let (_, Path { segments, .. }, _) = &trait_name;

    match segments.last() {
        Some(segment) => &segment.ident,
        None => unreachable!(),
    }
}

fn extract_contract_name(contract: &Type) -> &Ident {
    let Type::Path(type_path) = contract else {
        unreachable!()
    };
    let segments = &type_path.path.segments;
    assert!(!segments.is_empty());
    let segment = &segments.last().unwrap();
    &segment.ident
}

pub struct MultitestHelpers<'a> {
    error_type: Type,
    contract: &'a Type,
    source: &'a ItemImpl,
    generic_params: &'a [&'a GenericParam],
    generic_args: &'a [&'a GenericArgument],
    where_clause: &'a Option<syn::WhereClause>,
    contract_name: &'a Ident,
    proxy_name: Ident,
    custom: &'a Custom<'a>,
    override_entry_points: &'a OverrideEntryPoints,
    interfaces: &'a Interfaces,
    instantiate_variants: MsgVariants<'a, GenericParam>,
    exec_variants: MsgVariants<'a, GenericParam>,
    query_variants: MsgVariants<'a, GenericParam>,
    migrate_variants: MsgVariants<'a, GenericParam>,
    reply_variants: MsgVariants<'a, GenericParam>,
    contract_module: &'a Option<&'a Path>,
}

impl<'a> MultitestHelpers<'a> {
    pub fn new(
        source: &'a ItemImpl,
        generic_params: &'a [&'a GenericParam],
        generic_args: &'a [&'a GenericArgument],
        custom: &'a Custom,
        override_entry_points: &'a OverrideEntryPoints,
        interfaces: &'a Interfaces,
        contract_module: &'a Option<&'a Path>,
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

        let error_type: Type = if is_trait(source) {
            let associated_error = source.items.iter().find_map(|item| match item {
                ImplItem::Type(ty) if ty.ident == "Error" => Some(&ty.ty),
                _ => None,
            });
            match associated_error {
                Some(error) => parse_quote!(#error),
                // This should never happen as the `interface` macro requires the trait to have an associated `Error` type
                None => unreachable!(),
            }
        } else {
            let error = ContractErrorAttr::new(source).error;
            parse_quote! { #error }
        };

        let contract = &source.self_ty;
        let contract_name = extract_contract_name(contract);

        let proxy_name = if is_trait(source) {
            let interface_name = interface_name(source);
            Ident::new(&format!("{}Proxy", interface_name), interface_name.span())
        } else {
            Ident::new(&format!("{}Proxy", contract_name), contract_name.span())
        };

        Self {
            error_type,
            contract,
            source,
            generic_params,
            generic_args,
            where_clause,
            contract_name,
            proxy_name,
            custom,
            override_entry_points,
            interfaces,
            instantiate_variants,
            exec_variants,
            query_variants,
            migrate_variants,
            reply_variants,
            contract_module,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            source,
            error_type,
            proxy_name,
            custom,
            interfaces,
            exec_variants,
            query_variants,
            migrate_variants,
            generic_params,
            where_clause,
            ..
        } = self;
        let sylvia = crate_module();

        if is_trait(source) {
            return self.impl_trait_on_proxy();
        }

        let custom_msg = custom.msg_or_default();
        #[cfg(not(tarpaulin_include))]
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

        let exec_methods =
            exec_variants.emit_multitest_proxy_methods(&custom_msg, &mt_app, error_type);
        let query_methods =
            query_variants.emit_multitest_proxy_methods(&custom_msg, &mt_app, error_type);
        let migrate_methods =
            migrate_variants.emit_multitest_proxy_methods(&custom_msg, &mt_app, error_type);
        let where_predicates = where_clause
            .as_ref()
            .map(|where_clause| &where_clause.predicates);

        let contract_block = self.generate_contract_helpers();

        let proxy_accessors = interfaces.emit_proxy_accessors(&mt_app);

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod multitest_utils {
                    use super::*;
                    use #sylvia ::cw_multi_test::Executor;
                    use #sylvia ::derivative::Derivative;

                    #[derive(Derivative)]
                    #[derivative(Debug)]
                    pub struct #proxy_name <'app, MtApp, #(#generic_params,)* > {
                        pub contract_addr: #sylvia ::cw_std::Addr,
                        #[derivative(Debug="ignore")]
                        pub app: &'app #sylvia ::multitest::App<MtApp>,
                        _phantom: std::marker::PhantomData<( #(#generic_params,)* )>,
                    }

                    impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#generic_params,)* > #proxy_name <'app, #mt_app, #(#generic_params,)* >
                        where
                            CustomT: #sylvia ::cw_multi_test::Module,
                            CustomT::ExecT: std::fmt::Debug
                                + PartialEq
                                + Clone
                                + #sylvia ::schemars::JsonSchema
                                + #sylvia ::serde::de::DeserializeOwned
                                + 'static,
                            CustomT::QueryT: #sylvia ::cw_std::CustomQuery + #sylvia ::serde::de::DeserializeOwned + 'static,
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
                        pub fn new(contract_addr: #sylvia ::cw_std::Addr, app: &'app #sylvia ::multitest::App< #mt_app >) -> Self {
                            #proxy_name { contract_addr, app, _phantom: std::marker::PhantomData::default() }
                        }

                        #( #exec_methods )*
                        #( #migrate_methods )*
                        #( #query_methods )*
                        #( #proxy_accessors )*
                    }

                    impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#generic_params,)* >
                        From<(
                            #sylvia ::cw_std::Addr,
                            &'app #sylvia ::multitest::App<#mt_app>,
                        )>
                        for #proxy_name <'app, #mt_app, #(#generic_params,)* >
                    where
                        CustomT: #sylvia ::cw_multi_test::Module,
                        CustomT::ExecT: std::fmt::Debug
                            + PartialEq
                            + Clone
                            + #sylvia ::schemars::JsonSchema
                            + #sylvia ::serde::de::DeserializeOwned
                            + 'static,
                        CustomT::QueryT: #sylvia ::cw_std::CustomQuery + #sylvia ::serde::de::DeserializeOwned + 'static,
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
                        fn from(input: (#sylvia ::cw_std::Addr, &'app #sylvia ::multitest::App< #mt_app >))
                            -> #proxy_name<'app, #mt_app, #(#generic_params,)* > {
                            #proxy_name::new(input.0, input.1)
                        }
                    }

                    #contract_block
                }
            }
        }
    }

    fn impl_trait_on_proxy(&self) -> TokenStream {
        let Self {
            error_type,
            custom,
            interfaces,
            generic_params,
            generic_args,
            exec_variants,
            query_variants,
            where_clause,
            contract_module,
            contract_name,
            ..
        } = self;

        let sylvia = crate_module();

        let interface_name = interface_name(self.source);
        let proxy_name = &self.proxy_name;
        let trait_name = Ident::new(&format!("{}", interface_name), interface_name.span());

        let module = interfaces
            .get_only_interface()
            .map(|interface| {
                let module = &interface.module;
                quote! { #module :: }
            })
            .unwrap_or(quote! {});

        let custom_msg = custom.msg_or_default();

        #[cfg(not(tarpaulin_include))]
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

        let bracketed_generics = emit_bracketed_generics(generic_args);
        let interface_enum =
            quote! { < #module sv::Api #bracketed_generics as #sylvia ::types::InterfaceApi> };

        let exec_methods = exec_variants.emit_interface_multitest_proxy_methods(
            &custom_msg,
            &mt_app,
            error_type,
            generic_args,
            &module,
        );
        let query_methods = query_variants.emit_interface_multitest_proxy_methods(
            &custom_msg,
            &mt_app,
            error_type,
            generic_args,
            &module,
        );
        let exec_methods_declarations =
            exec_variants.emit_proxy_methods_declarations(&custom_msg, error_type, &interface_enum);
        let query_methods_declarations = query_variants.emit_proxy_methods_declarations(
            &custom_msg,
            error_type,
            &interface_enum,
        );
        let exec_generics = exec_variants.used_generics();
        let query_generics = query_variants.used_generics();

        // Check if custom_msg is a generic type

        let mut generics_checker = CheckGenerics::new(generic_params);
        generics_checker.visit_type(&custom_msg);
        let (custom_generic, _) = generics_checker.used_unused();
        let custom_generic = custom_generic.first();
        let custom_where_predicate = match custom_generic {
            Some(generic) => filter_wheres(where_clause, generic_params, &[generic]),
            None => vec![],
        };
        let exec_where_predicates = filter_wheres(where_clause, generic_params, exec_generics);
        let query_where_predicates = filter_wheres(where_clause, generic_params, query_generics);
        let trait_where_clause = if !exec_where_predicates.is_empty() {
            quote! { where #(#exec_where_predicates,)* }
        } else {
            quote! {}
        };
        let contract_module = match contract_module {
            Some(contract_module) => quote! { #contract_module :: },
            None => quote! {},
        };
        let contract_proxy = Ident::new(&format!("{}Proxy", contract_name), contract_name.span());
        let contract_generics: Vec<_> = exec_generics
            .iter()
            .chain(query_generics.iter())
            .chain(generic_params.iter())
            .unique()
            .collect();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod test_utils {
                    use super::*;

                    pub trait #trait_name<MtApp, #(#exec_generics,)* #(#query_generics,)* #custom_generic > #trait_where_clause {
                        #(#query_methods_declarations)*
                        #(#exec_methods_declarations)*
                    }

                    impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#exec_generics,)* #(#query_generics,)* #custom_generic> #trait_name< #mt_app , #(#exec_generics,)* #(#query_generics,)* #custom_generic > for #module sv::trait_utils:: #proxy_name<'_, #mt_app >
                    where
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
                        CustomT::ExecT: Clone
                            + std::fmt::Debug
                            + PartialEq
                            + #sylvia ::schemars::JsonSchema
                            + #sylvia ::serde::de::DeserializeOwned
                            + 'static,
                        CustomT::QueryT: #sylvia:: cw_std::CustomQuery + #sylvia ::serde::de::DeserializeOwned + 'static,
                        #mt_app : #sylvia ::cw_multi_test::Executor< #custom_msg >,
                        #(#custom_where_predicate,)*
                        #(#exec_where_predicates,)*
                        #(#query_where_predicates,)*
                    {
                        #(#query_methods)*
                        #(#exec_methods)*
                    }

                    impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#contract_generics,)* > #trait_name< #mt_app , #(#exec_generics,)* #(#query_generics,)* #custom_generic > for #contract_module sv::multitest_utils:: #contract_proxy <'_, #mt_app, #(#contract_generics,)* >
                    where
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
                        CustomT::ExecT: Clone
                            + std::fmt::Debug
                            + PartialEq
                            + #sylvia ::schemars::JsonSchema
                            + #sylvia ::serde::de::DeserializeOwned
                            + 'static,
                        CustomT::QueryT: #sylvia:: cw_std::CustomQuery + #sylvia ::serde::de::DeserializeOwned + 'static,
                        #mt_app : #sylvia ::cw_multi_test::Executor< #custom_msg >,
                        #(#custom_where_predicate,)*
                        #(#exec_where_predicates,)*
                        #(#query_where_predicates,)*
                    {
                        #(#query_methods)*
                        #(#exec_methods)*
                    }
                }
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
            proxy_name,
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
        let contract = if !generic_params.is_empty() {
            quote! { #contract_name ::< #(#generic_params,)* > }
        } else {
            quote! { #contract_name }
        };

        let instantiate_msg = if !used_generics.is_empty() {
            quote! { InstantiateMsg::< #(#used_generics,)* > }
        } else {
            quote! { InstantiateMsg }
        };

        let impl_contract = self.generate_impl_contract();

        let custom_msg = self.custom.msg_or_default();
        let custom_query = self.custom.query_or_default();

        #[cfg(not(tarpaulin_include))]
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
            #[cfg(not(tarpaulin_include))]
            {
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
                        .map(|data| #proxy_name {
                            contract_addr: #sylvia ::cw_std::Addr::unchecked(data.contract_address),
                            app: code_id.app,
                            _phantom: std::marker::PhantomData::default(),
                        })
                }
            }
        } else {
            #[cfg(not(tarpaulin_include))]
            {
                quote! {
                    let err = #sylvia ::cw_std::StdError::generic_err(
                        "`with_salt` was called, but it requires `cosmwasm_1_2` feature enabled. Consider removing `with_salt` or adding the `cosmwasm_1_2` feature."
                    );
                    Err(Into::into(err))
                }
            }
        };

        #[cfg(not(tarpaulin_include))]
        let code_info = if cfg!(feature = "cosmwasm_1_2") {
            quote! {
                pub fn code_info(&self) -> #sylvia ::cw_std::StdResult< #sylvia ::cw_std::CodeInfoResponse> {
                    self.app.app().wrap().query_wasm_code_info(self.code_id)
                }
            }
        } else {
            quote! {}
        };

        #[cfg(not(tarpaulin_include))]
        {
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
                    pub fn call(self, sender: &str) -> Result<#proxy_name<'app, MtApp, #(#generic_params,)* >, #error_type> {
                        let Self {code_id, funds, label, admin, salt, msg} = self;

                        match salt {
                            Some(salt) => {
                                #instantiate2
                            },
                            None => (*code_id.app)
                                .app_mut()
                                .instantiate_contract(
                                    code_id.code_id,
                                    #sylvia ::cw_std::Addr::unchecked(sender),
                                    &msg,
                                    funds,
                                    label,
                                    admin,
                                )
                                .map_err(|err| err.downcast().unwrap())
                                .map(|addr| #proxy_name {
                                    contract_addr: addr,
                                    app: code_id.app,
                                    _phantom: std::marker::PhantomData::default(),
                                }),
                        }
                    }
                }
            }
        }
    }

    fn generate_impl_contract(&self) -> TokenStream {
        let Self {
            source,
            contract,
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
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Instantiate, contract));

        let exec_body = override_entry_points
            .get_entry_point(MsgType::Exec)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Exec, contract));

        let query_body = override_entry_points
            .get_entry_point(MsgType::Query)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| emit_default_dispatch(&MsgType::Query, contract));

        let sudo_body = override_entry_points
            .get_entry_point(MsgType::Sudo)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| {
                quote! {
                    #sylvia ::anyhow::bail!("sudo not implemented for contract")
                }
            });

        let migrate_body = match override_entry_points.get_entry_point(MsgType::Migrate) {
            Some(entry_point) => entry_point.emit_multitest_dispatch(),
            None if migrate_variants.get_only_variant().is_some() => {
                emit_default_dispatch(&MsgType::Migrate, contract)
            }
            None => quote! {
                #sylvia ::anyhow::bail!("migrate not implemented for contract")
            },
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

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                impl #bracketed_generics #sylvia ::cw_multi_test::Contract<#custom_msg, #custom_query> for #contract #full_where_clause {
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
}

fn emit_default_dispatch(msg_ty: &MsgType, contract_name: &Type) -> TokenStream {
    let sylvia = crate_module();

    let values = msg_ty.emit_ctx_values();
    let msg_name = msg_ty.as_accessor_name(true);
    let api_msg = quote! { < #contract_name as #sylvia ::types::ContractApi> :: #msg_name };

    quote! {
        #sylvia ::cw_std::from_json::< #api_msg >(&msg)?
            .dispatch(self, ( #values ))
            .map_err(Into::into)
    }
}

pub struct TraitMultitestHelpers<'a> {
    trait_name: &'a Ident,
}

impl<'a> TraitMultitestHelpers<'a> {
    pub fn new(source: &'a ItemTrait) -> Self {
        Self {
            trait_name: &source.ident,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let trait_name = self.trait_name;
        let sylvia = crate_module();
        let proxy_name = Ident::new(&format!("{}Proxy", trait_name), trait_name.span());

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod trait_utils {
                    pub struct #proxy_name <'app, MtApp> {
                        pub contract_addr: #sylvia ::cw_std::Addr,
                        pub app: &'app #sylvia ::multitest::App <MtApp>,
                    }
                    impl<'app, MtApp> #proxy_name <'app, MtApp> {
                        pub fn new(contract_addr: #sylvia ::cw_std::Addr, app: &'app #sylvia ::multitest::App < MtApp >) -> Self {
                            #proxy_name { contract_addr, app }
                        }
                    }
                    #[allow(clippy::from_over_into)]
                    impl<MtApp> Into<#sylvia ::cw_std::Addr> for #proxy_name <'_, MtApp> {
                        fn into(self) -> #sylvia ::cw_std::Addr {
                            self.contract_addr
                        }
                    }
                }
            }
        }
    }
}
