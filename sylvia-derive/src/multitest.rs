use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, ImplItem, ItemImpl, ItemTrait, Path, Type};

use crate::check_generics::GetPath;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::message::{MsgVariant, MsgVariants};
use crate::parser::{Custom, MsgType, OverrideEntryPoint, OverrideEntryPoints};
use crate::utils::emit_bracketed_generics;
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
    let segment = &segments[0];
    &segment.ident
}

pub struct MultitestHelpers<'a, Generics> {
    error_type: Type,
    contract: &'a Type,
    is_trait: bool,
    source: &'a ItemImpl,
    generics: &'a [&'a Generics],
    where_clause: &'a Option<syn::WhereClause>,
    contract_name: &'a Ident,
    proxy_name: Ident,
    custom: &'a Custom<'a>,
    override_entry_points: &'a OverrideEntryPoints,
    interfaces: &'a Interfaces,
    instantiate_variants: MsgVariants<'a, Generics>,
    exec_variants: MsgVariants<'a, Generics>,
    query_variants: MsgVariants<'a, Generics>,
    migrate_variants: MsgVariants<'a, Generics>,
    reply_variants: MsgVariants<'a, Generics>,
}

impl<'a, Generics> MultitestHelpers<'a, Generics>
where
    Generics: ToTokens + PartialEq + GetPath,
{
    pub fn new(
        source: &'a ItemImpl,
        is_trait: bool,
        contract_error: &'a Type,
        generics: &'a [&'a Generics],
        custom: &'a Custom,
        override_entry_points: &'a OverrideEntryPoints,
        interfaces: &'a Interfaces,
    ) -> Self {
        let where_clause = &source.generics.where_clause;
        let instantiate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Instantiate,
            generics,
            where_clause,
        );
        let exec_variants =
            MsgVariants::new(source.as_variants(), MsgType::Exec, generics, where_clause);
        let query_variants =
            MsgVariants::new(source.as_variants(), MsgType::Query, generics, where_clause);
        let migrate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Migrate,
            generics,
            where_clause,
        );
        let reply_variants =
            MsgVariants::new(source.as_variants(), MsgType::Reply, generics, where_clause);

        let error_type: Type = if is_trait {
            let error_type: Vec<_> = source
                .items
                .iter()
                .filter_map(|item| match item {
                    ImplItem::Type(ty) if ty.ident == "Error" => {
                        let ty = &ty.ty;
                        let segments = match ty {
                            Type::Path(path) => &path.path.segments,
                            _ => {
                                unreachable!();
                            }
                        };
                        assert!(!segments.is_empty());

                        Some(&segments[0].ident)
                    }
                    _ => None,
                })
                .collect();

            assert!(!error_type.is_empty());
            let error_type = error_type[0];
            parse_quote! {#error_type}
        } else {
            parse_quote! {#contract_error}
        };

        let contract = &source.self_ty;
        let contract_name = extract_contract_name(contract);

        let proxy_name = if is_trait {
            let interface_name = interface_name(source);
            Ident::new(&format!("{}Proxy", interface_name), interface_name.span())
        } else {
            Ident::new(&format!("{}Proxy", contract_name), contract_name.span())
        };

        Self {
            error_type,
            contract,
            is_trait,
            source,
            generics,
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
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            error_type,
            proxy_name,
            is_trait,
            custom,
            interfaces,
            exec_variants,
            query_variants,
            migrate_variants,
            generics,
            where_clause,
            ..
        } = self;
        let sylvia = crate_module();

        if *is_trait {
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
                    pub struct #proxy_name <'app, MtApp, #(#generics,)* > {
                        pub contract_addr: #sylvia ::cw_std::Addr,
                        #[derivative(Debug="ignore")]
                        pub app: &'app #sylvia ::multitest::App<MtApp>,
                        _phantom: std::marker::PhantomData<( #(#generics,)* )>,
                    }

                    impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#generics,)* > #proxy_name <'app, #mt_app, #(#generics,)* >
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

                    impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT, #(#generics,)* >
                        From<(
                            #sylvia ::cw_std::Addr,
                            &'app #sylvia ::multitest::App<#mt_app>,
                        )>
                        for #proxy_name <'app, #mt_app, #(#generics,)* >
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
                            -> #proxy_name<'app, #mt_app, #(#generics,)* > {
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
            generics,
            exec_variants,
            query_variants,
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

        let bracketed_generics = emit_bracketed_generics(generics);
        let interface_enum =
            quote! { < #module sv::Api #bracketed_generics as #sylvia ::types::InterfaceApi> };

        let exec_methods = exec_variants.emit_interface_multitest_proxy_methods(
            &custom_msg,
            &mt_app,
            error_type,
            generics,
            &module,
        );
        let query_methods = query_variants.emit_interface_multitest_proxy_methods(
            &custom_msg,
            &mt_app,
            error_type,
            generics,
            &module,
        );
        let exec_methods_declarations =
            exec_variants.emit_proxy_methods_declarations(&custom_msg, error_type, &interface_enum);
        let query_methods_declarations = query_variants.emit_proxy_methods_declarations(
            &custom_msg,
            error_type,
            &interface_enum,
        );

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod test_utils {
                    use super::*;

                    pub trait #trait_name<MtApp> {
                        #(#query_methods_declarations)*
                        #(#exec_methods_declarations)*
                    }

                    impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT> #trait_name< #mt_app > for #module sv::trait_utils:: #proxy_name<'_, #mt_app >
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
                        #mt_app : #sylvia ::cw_multi_test::Executor< #custom_msg >
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
            is_trait,
            generics,
            where_clause,
            contract_name,
            proxy_name,
            instantiate_variants,
            ..
        } = self;

        if *is_trait {
            return quote! {};
        }

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
        let contract = if !generics.is_empty() {
            quote! { #contract_name ::< #(#generics,)* > }
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

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #impl_contract

                pub struct CodeId<'app, #(#generics,)* MtApp> {
                    code_id: u64,
                    app: &'app #sylvia ::multitest::App<MtApp>,
                    _phantom: std::marker::PhantomData<( #(#generics,)* )>,

                }

                impl<'app, BankT, ApiT, StorageT, CustomT, StakingT, DistrT, IbcT, GovT, #(#generics,)* > CodeId<'app, #(#generics,)* #mt_app >
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

                    pub fn instantiate(
                        &self,#(#fields,)*
                    ) -> InstantiateProxy<'_, 'app, #(#generics,)* #mt_app > {
                        let msg = #instantiate_msg {#(#fields_names,)*};
                        InstantiateProxy::< #(#generics,)* _> {
                            code_id: self,
                            funds: &[],
                            label: "Contract",
                            admin: None,
                            msg,
                        }
                    }
                }

                pub struct InstantiateProxy<'proxy, 'app, #(#generics,)* MtApp> {
                    code_id: &'proxy CodeId <'app, #(#generics,)* MtApp>,
                    funds: &'proxy [#sylvia ::cw_std::Coin],
                    label: &'proxy str,
                    admin: Option<String>,
                    msg: InstantiateMsg #bracketed_used_generics,
                }

                impl<'proxy, 'app, #(#generics,)* MtApp> InstantiateProxy<'proxy, 'app, #(#generics,)* MtApp>
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

                    #[track_caller]
                    pub fn call(self, sender: &str) -> Result<#proxy_name<'app, MtApp, #(#generics,)* >, #error_type> {
                        (*self.code_id.app)
                            .app_mut()
                            .instantiate_contract(
                                self.code_id.code_id,
                                #sylvia ::cw_std::Addr::unchecked(sender),
                                &self.msg,
                                self.funds,
                                self.label,
                                self.admin,
                            )
                            .map_err(|err| err.downcast().unwrap())
                            .map(|addr| #proxy_name {
                                contract_addr: addr,
                                app: self.code_id.app,
                                _phantom: std::marker::PhantomData::default(),
                            })
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
            generics,
            instantiate_variants,
            exec_variants,
            query_variants,
            migrate_variants,
            reply_variants,
            ..
        } = self;
        let sylvia = crate_module();

        let bracketed_generics = emit_bracketed_generics(generics);
        let full_where_clause = &source.generics.where_clause;
        let instantiate_body = override_entry_points
            .get_entry_point(MsgType::Instantiate)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| instantiate_variants.emit_multitest_default_dispatch());

        let exec_body = override_entry_points
            .get_entry_point(MsgType::Exec)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| exec_variants.emit_multitest_default_dispatch());

        let query_body = override_entry_points
            .get_entry_point(MsgType::Query)
            .map(OverrideEntryPoint::emit_multitest_dispatch)
            .unwrap_or_else(|| query_variants.emit_multitest_default_dispatch());

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
                migrate_variants.emit_multitest_default_dispatch()
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
