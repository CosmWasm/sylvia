use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{parse_quote, FnArg, ImplItem, ItemImpl, ItemTrait, Pat, PatType, Path, Type};

use crate::check_generics::GetPath;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::message::{MsgVariant, MsgVariants};
use crate::parser::{Custom, MsgAttr, MsgType, OverrideEntryPoint, OverrideEntryPoints};
use crate::utils::{emit_bracketed_generics, extract_return_type};
use crate::variant_descs::AsVariantDescs;

fn interface_name(source: &ItemImpl) -> &Ident {
    let trait_name = &source.trait_;
    let Some(trait_name) = trait_name else {
        unreachable!()
    };
    let (_, Path { segments, .. }, _) = &trait_name;
    assert!(!segments.is_empty());

    &segments[0].ident
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

struct MessageSignature<'a> {
    pub name: &'a Ident,
    pub params: Vec<TokenStream>,
    pub arguments: Vec<&'a Ident>,
    pub msg_ty: MsgType,
    pub return_type: TokenStream,
}

pub struct MultitestHelpers<'a, Generics> {
    messages: Vec<MessageSignature<'a>>,
    error_type: TokenStream,
    contract: &'a Type,
    is_trait: bool,
    reply: Option<Ident>,
    source: &'a ItemImpl,
    generics: &'a [&'a Generics],
    contract_name: &'a Ident,
    proxy_name: Ident,
    custom: &'a Custom<'a>,
    override_entry_points: &'a OverrideEntryPoints,
    interfaces: &'a Interfaces,
    instantiate_variants: MsgVariants<'a, Generics>,
    exec_variants: MsgVariants<'a, Generics>,
    query_variants: MsgVariants<'a, Generics>,
    migrate_variants: MsgVariants<'a, Generics>,
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
        let mut reply = None;
        let sylvia = crate_module();

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

        let messages: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Method(method) => {
                    let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                    let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                        Ok(attr) => attr,
                        Err(err) => {
                            emit_error!(method.span(), err);
                            return None;
                        }
                    };
                    let msg_ty = attr.msg_type();

                    if msg_ty == MsgType::Reply {
                        reply = Some(method.sig.ident.clone());
                        return None;
                    } else if ![MsgType::Query, MsgType::Exec, MsgType::Migrate].contains(&msg_ty) {
                        return None;
                    }

                    let sig = &method.sig;
                    let return_type = if let MsgAttr::Query { resp_type } = attr {
                        match resp_type {
                            Some(resp_type) => quote! {#resp_type},
                            None => {
                                let return_type = extract_return_type(&sig.output);
                                quote! {#return_type}
                            }
                        }
                    } else {
                        quote! { #sylvia ::cw_multi_test::AppResponse }
                    };

                    let name = &sig.ident;
                    let params: Vec<_> = sig
                        .inputs
                        .iter()
                        .skip(2)
                        .filter_map(|arg| match arg {
                            FnArg::Typed(ty) => {
                                let name = match ty.pat.as_ref() {
                                    Pat::Ident(ident) => &ident.ident,
                                    _ => return None,
                                };
                                let ty = &ty.ty;
                                Some(quote! {#name : #ty})
                            }
                            _ => None,
                        })
                        .collect();
                    let arguments: Vec<_> = sig
                        .inputs
                        .iter()
                        .skip(2)
                        .filter_map(|arg| match arg {
                            FnArg::Typed(item) => {
                                let PatType { pat, .. } = item;
                                let Pat::Ident(ident) = pat.as_ref() else {
                                    unreachable!()
                                };
                                Some(&ident.ident)
                            }
                            _ => None,
                        })
                        .collect();

                    Some(MessageSignature {
                        name,
                        params,
                        arguments,
                        msg_ty,
                        return_type,
                    })
                }
                _ => None,
            })
            .collect();

        let error_type = if is_trait {
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
            quote! {#error_type}
        } else {
            quote! {#contract_error}
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
            messages,
            error_type,
            contract,
            is_trait,
            reply,
            source,
            generics,
            contract_name,
            proxy_name,
            custom,
            override_entry_points,
            interfaces,
            instantiate_variants,
            exec_variants,
            query_variants,
            migrate_variants,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            messages,
            error_type,
            proxy_name,
            is_trait,
            custom,
            interfaces,
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

        #[cfg(not(tarpaulin_include))]
        let messages = messages.iter().map(|msg| {
            let MessageSignature {
                name,
                params,
                arguments,
                msg_ty,
                return_type,
            } = msg;
            if msg_ty == &MsgType::Exec {
                    quote! {
                        #[track_caller]
                        pub fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, ExecMsg, #mt_app, #custom_msg> {
                            let msg = ExecMsg:: #name ( #(#arguments),* );

                            #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                        }
                }
            } else if msg_ty == &MsgType::Migrate {
                    quote! {
                        #[track_caller]
                        pub fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::MigrateProxy::<#error_type, MigrateMsg, #mt_app, #custom_msg> {
                            let msg = MigrateMsg::new( #(#arguments),* );

                            #sylvia ::multitest::MigrateProxy::new(&self.contract_addr, msg, &self.app)
                    }
                }
            } else if msg_ty == &MsgType::Query {
                    quote! {
                        pub fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                            let msg = QueryMsg:: #name ( #(#arguments),* );

                            (*self.app)
                                .app()
                                .wrap()
                                .query_wasm_smart(self.contract_addr.clone(), &msg)
                                .map_err(Into::into)
                        }
                    }
            } else {
                quote! {}
            }
        });

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
                    pub struct #proxy_name <'app, MtApp> {
                        pub contract_addr: #sylvia ::cw_std::Addr,
                        #[derivative(Debug="ignore")]
                        pub app: &'app #sylvia ::multitest::App<MtApp>,
                    }

                    impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT> #proxy_name <'app, #mt_app >
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
                            #mt_app : Executor< #custom_msg >
                    {
                        pub fn new(contract_addr: #sylvia ::cw_std::Addr, app: &'app #sylvia ::multitest::App< #mt_app >) -> Self {
                            #proxy_name{ contract_addr, app }
                        }

                        #(#messages)*

                        #(#proxy_accessors)*
                    }

                    impl<'app, BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT>
                        From<(
                            #sylvia ::cw_std::Addr,
                            &'app #sylvia ::multitest::App<#mt_app>,
                        )>
                        for #proxy_name <'app, #mt_app >
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
                    {
                        fn from(input: (#sylvia ::cw_std::Addr, &'app #sylvia ::multitest::App< #mt_app >))
                            -> #proxy_name<'app, #mt_app > {
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
            messages,
            error_type,
            custom,
            interfaces,
            generics,
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
        let mt_app = quote! {
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
        let interface_enum = quote! { < #module InterfaceTypes #bracketed_generics as #sylvia ::types::InterfaceMessages> };

        #[cfg(not(tarpaulin_include))]
        let methods_definitions = messages.iter().map(|msg| {
            let MessageSignature {
                name,
                params,
                arguments,
                msg_ty,
                return_type,
            } = msg;
        let type_name = msg_ty.as_accessor_name();
            if msg_ty == &MsgType::Exec {
                quote! {
                    #[track_caller]
                    fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #interface_enum :: #type_name, #mt_app, #custom_msg> {
                        let msg = #interface_enum :: #type_name :: #name ( #(#arguments),* );

                        #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                    }
                }
            } else {
                quote! {
                    fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                        let msg = #interface_enum :: #type_name :: #name ( #(#arguments),* );

                        (*self.app)
                            .app()
                            .wrap()
                            .query_wasm_smart(self.contract_addr.clone(), &msg)
                            .map_err(Into::into)
                    }
                }
            }
        });

        #[cfg(not(tarpaulin_include))]
        let methods_declarations = messages.iter().map(|msg| {
            let MessageSignature {
                name,
                params,
                msg_ty,
                return_type,
                ..
            } = msg;
            let type_name = msg_ty.as_accessor_name();
            if msg_ty == &MsgType::Exec {
                quote! {
                    fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #interface_enum :: #type_name, MtApp, #custom_msg>;
                }
            } else {
                quote! {
                    fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type>;
                }
            }
        });

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod test_utils {
                    use super::*;

                    pub trait #trait_name<MtApp> {
                        #(#methods_declarations)*
                    }

                    impl<BankT, ApiT, StorageT, CustomT, WasmT, StakingT, DistrT, IbcT, GovT> #trait_name< #mt_app > for #module trait_utils:: #proxy_name<'_, #mt_app >
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

                        #(#methods_definitions)*
                    }
                }
            }
        }
    }

    fn generate_contract_helpers(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            error_type,
            is_trait,
            generics,
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
        let bracketed_generics = emit_bracketed_generics(generics);
        let full_where_clause = &source.generics.where_clause;

        let where_predicates = instantiate_variants.where_predicates();
        let where_clause = instantiate_variants.where_clause();
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

                pub struct CodeId<'app, MtApp> {
                    code_id: u64,
                    app: &'app #sylvia ::multitest::App<MtApp>,
                }

                impl<'app, BankT, ApiT, StorageT, CustomT, StakingT, DistrT, IbcT, GovT> CodeId<'app, #mt_app>
                    where
                        BankT: #sylvia ::cw_multi_test::Bank,
                        ApiT: #sylvia ::cw_std::Api,
                        StorageT: #sylvia ::cw_std::Storage,
                        CustomT: #sylvia ::cw_multi_test::Module<ExecT = #custom_msg, QueryT = #custom_query >,
                        StakingT: #sylvia ::cw_multi_test::Staking,
                        DistrT: #sylvia ::cw_multi_test::Distribution,
                        IbcT: #sylvia ::cw_multi_test::Ibc,
                        GovT: #sylvia ::cw_multi_test::Gov,
                {
                    pub fn store_code #bracketed_generics (app: &'app #sylvia ::multitest::App< #mt_app >) -> Self #full_where_clause {
                        let code_id = app
                            .app_mut()
                            .store_code(Box::new(#contract ::new()));
                        Self { code_id, app }
                    }

                    pub fn code_id(&self) -> u64 {
                        self.code_id
                    }

                    pub fn instantiate #bracketed_used_generics (
                        &self,#(#fields,)*
                    ) -> InstantiateProxy<'_, 'app, #mt_app, #(#used_generics,)* > #where_clause {
                        let msg = #instantiate_msg {#(#fields_names,)*};
                        InstantiateProxy::<_, #(#used_generics,)* > {
                            code_id: self,
                            funds: &[],
                            label: "Contract",
                            admin: None,
                            msg,
                        }
                    }
                }

                pub struct InstantiateProxy<'a, 'app, MtApp, #(#used_generics,)* > {
                    code_id: &'a CodeId <'app, MtApp>,
                    funds: &'a [#sylvia ::cw_std::Coin],
                    label: &'a str,
                    admin: Option<String>,
                    msg: InstantiateMsg #bracketed_used_generics,
                }

                impl<'a, 'app, MtApp, #(#used_generics,)* > InstantiateProxy<'a, 'app, MtApp, #(#used_generics,)* >
                    where
                        MtApp: Executor< #custom_msg >,
                        #(#where_predicates,)*
                {
                    pub fn with_funds(self, funds: &'a [#sylvia ::cw_std::Coin]) -> Self {
                        Self { funds, ..self }
                    }

                    pub fn with_label(self, label: &'a str) -> Self {
                        Self { label, ..self }
                    }

                    pub fn with_admin<'s>(self, admin: impl Into<Option<&'s str>>) -> Self {
                        let admin = admin.into().map(str::to_owned);
                        Self { admin, ..self }
                    }

                    #[track_caller]
                    pub fn call(self, sender: &str) -> Result<#proxy_name<'app, MtApp>, #error_type> {
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
            None => self
                .reply
                .as_ref()
                .map(|reply| {
                    quote! {
                        self. #reply((deps, env).into(), msg).map_err(Into::into)
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
