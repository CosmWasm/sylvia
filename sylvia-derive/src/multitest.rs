use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{FnArg, GenericParam, ImplItem, ItemImpl, ItemTrait, Pat, PatType, Path, Type};

use crate::check_generics::CheckGenerics;
use crate::crate_module;
use crate::message::MsgField;
use crate::parser::{parse_struct_message, ContractMessageAttr, Custom, MsgAttr, MsgType};
use crate::utils::{extract_return_type, process_fields};

struct MessageSignature<'a> {
    pub name: &'a Ident,
    pub params: Vec<TokenStream>,
    pub arguments: Vec<&'a Ident>,
    pub msg_ty: MsgType,
    pub return_type: TokenStream,
}

pub struct MultitestHelpers<'a> {
    messages: Vec<MessageSignature<'a>>,
    error_type: TokenStream,
    contract: &'a Type,
    is_trait: bool,
    is_migrate: bool,
    reply: Option<Ident>,
    source: &'a ItemImpl,
    generics: &'a [&'a GenericParam],
    contract_name: &'a Ident,
    proxy_name: Ident,
    custom: &'a Custom,
}

fn interface_name(source: &ItemImpl) -> &Ident {
    let trait_name = &source.trait_;
    let Some(trait_name) = trait_name else {unreachable!()};
    let (_, Path { segments, .. }, _) = &trait_name;
    assert!(!segments.is_empty());

    &segments[0].ident
}

fn extract_contract_name(contract: &Type) -> &Ident {
    let Type::Path(type_path) = contract  else {
            unreachable!()
        };
    let segments = &type_path.path.segments;
    assert!(!segments.is_empty());
    let segment = &segments[0];
    &segment.ident
}

impl<'a> MultitestHelpers<'a> {
    pub fn new(
        source: &'a ItemImpl,
        is_trait: bool,
        contract_error: &'a Type,
        generics: &'a [&'a GenericParam],
        custom: &'a Custom,
    ) -> Self {
        let mut is_migrate = false;
        let mut reply = None;
        let sylvia = crate_module();

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

                    if msg_ty == MsgType::Migrate {
                        is_migrate = true;
                    } else if msg_ty == MsgType::Reply {
                        reply = Some(method.sig.ident.clone());
                        return None;
                    } else if msg_ty != MsgType::Query && msg_ty != MsgType::Exec {
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
            is_migrate,
            reply,
            source,
            generics,
            contract_name,
            proxy_name,
            custom,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            messages,
            error_type,
            proxy_name,
            source,
            is_trait,
            custom,
            ..
        } = self;
        let sylvia = crate_module();

        if *is_trait {
            return self.impl_trait_on_proxy();
        }

        let custom_msg = custom.msg();
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

        let interfaces: Vec<_> = source
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("messages"))
            .filter_map(|attr| {
                let interface = match ContractMessageAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(interface) => {
                        let ContractMessageAttr { module, .. } = interface;
                        assert!(!module.segments.is_empty());
                        let module_name = &module.segments[0].ident;
                        let method_name = Ident::new(&format!("{}_proxy", module_name), module_name.span());
                        let proxy_name = Ident::new(
                            &format!("{}Proxy", module_name.to_string().to_case(Case::UpperCamel)),
                            module_name.span(),
                        );

                        quote! {
                            pub fn #method_name (&self) -> #module ::trait_utils:: #proxy_name <'app, #mt_app> {
                                #module ::trait_utils:: #proxy_name ::new(self.contract_addr.clone(), self.app)
                            }
                        }
                    }
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        return None;
                    }
                };

                Some(interface)
            })
            .collect();

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

                        #(#interfaces)*
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
            source,
            custom,
            ..
        } = self;

        let sylvia = crate_module();

        let interface_name = interface_name(self.source);
        let proxy_name = &self.proxy_name;
        let trait_name = Ident::new(&format!("{}", interface_name), interface_name.span());

        let modules: Vec<_> = source
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("messages"))
            .filter_map(
                |attr| match ContractMessageAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(interface) => {
                        let ContractMessageAttr { module, .. } = &interface;
                        assert!(!module.segments.is_empty());
                        Some(module.segments[0].ident.clone())
                    }
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        None
                    }
                },
            )
            .collect();

        #[cfg(not(tarpaulin_include))]
        let module = match modules.len() {
            0 => {
                quote! {}
            }
            1 => {
                let module = &modules[0];
                quote! {#module ::}
            }
            _ => {
                emit_error!(
                    source.span(),
                    "Only one #[messages] attribute is allowed per contract"
                );
                return quote! {};
            }
        };

        let custom_msg = custom.msg();

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

        #[cfg(not(tarpaulin_include))]
        let methods_definitions = messages.iter().map(|msg| {
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
                    fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #module ExecMsg, #mt_app, #custom_msg> {
                        let msg = #module ExecMsg:: #name ( #(#arguments),* );

                        #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                    }
                }
            } else {
                quote! {
                    fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                        let msg = #module QueryMsg:: #name ( #(#arguments),* );

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
            if msg_ty == &MsgType::Exec {
                quote! {
                    fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, #module ExecMsg, MtApp, #custom_msg>;
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
        let Self {
            error_type,
            is_trait,
            source,
            generics,
            contract_name,
            proxy_name,
            ..
        } = self;

        if *is_trait {
            return quote! {};
        }

        let sylvia = crate_module();

        let mut generics_checker = CheckGenerics::new(generics);

        let parsed = parse_struct_message(source, MsgType::Instantiate);
        let Some((method,_)) = parsed else {
            return quote! {};
        };

        let instantiate_fields = process_fields(&method.sig, &mut generics_checker);
        let fields_names: Vec<_> = instantiate_fields.iter().map(MsgField::name).collect();
        let fields = instantiate_fields.iter().map(MsgField::emit);

        let impl_contract = self.generate_impl_contract();

        let custom_msg = self.custom.msg();

        #[cfg(not(tarpaulin_include))]
        let mt_app = quote! {
            #sylvia ::cw_multi_test::App<
                BankT,
                ApiT,
                StorageT,
                CustomT,
                #sylvia ::cw_multi_test::WasmKeeper< #custom_msg , #sylvia ::cw_std::Empty>,
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
                        CustomT: #sylvia ::cw_multi_test::Module<ExecT = #custom_msg, QueryT = #sylvia ::cw_std::Empty >,
                        StakingT: #sylvia ::cw_multi_test::Staking,
                        DistrT: #sylvia ::cw_multi_test::Distribution,
                        IbcT: #sylvia ::cw_multi_test::Ibc,
                        GovT: #sylvia ::cw_multi_test::Gov,
                {
                    pub fn store_code(app: &'app #sylvia ::multitest::App< #mt_app >) -> Self {
                        let code_id = app
                            .app_mut()
                            .store_code(Box::new(#contract_name ::new()));
                        Self { code_id, app }
                    }

                    pub fn code_id(&self) -> u64 {
                        self.code_id
                    }

                    pub fn instantiate(
                        &self,#(#fields,)*
                    ) -> InstantiateProxy<'_, 'app, #mt_app > {
                        let msg = InstantiateMsg {#(#fields_names,)*};
                        InstantiateProxy {
                            code_id: self,
                            funds: &[],
                            label: "Contract",
                            admin: None,
                            msg,
                        }
                    }
                }

                pub struct InstantiateProxy<'a, 'app, MtApp> {
                    code_id: &'a CodeId <'app, MtApp>,
                    funds: &'a [#sylvia ::cw_std::Coin],
                    label: &'a str,
                    admin: Option<String>,
                    msg: InstantiateMsg,
                }

                impl<'a, 'app, MtApp> InstantiateProxy<'a, 'app, MtApp>
                    where
                        MtApp: Executor< #custom_msg >,
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
            contract, custom, ..
        } = self;
        let sylvia = crate_module();

        #[cfg(not(tarpaulin_include))]
        let migrate_body = if self.is_migrate {
            quote! {
                #sylvia ::cw_std::from_slice::<MigrateMsg>(&msg)?
                    .dispatch(self, (deps, env).into())
                    .map_err(Into::into)
            }
        } else {
            quote! {
                #sylvia ::anyhow::bail!("migrate not implemented for contract")
            }
        };

        #[cfg(not(tarpaulin_include))]
        let reply = if let Some(reply) = self.reply.as_ref() {
            quote! {
                self. #reply((deps, env).into(), msg).map_err(Into::into)
            }
        } else {
            quote! {
                #sylvia ::anyhow::bail!("reply not implemented for contract")
            }
        };

        let custom_msg = custom.msg();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                impl #sylvia ::cw_multi_test::Contract<#custom_msg> for #contract {
                    fn execute(
                        &self,
                        deps: #sylvia ::cw_std::DepsMut<#sylvia ::cw_std::Empty>,
                        env: #sylvia ::cw_std::Env,
                        info: #sylvia ::cw_std::MessageInfo,
                        msg: Vec<u8>,
                    ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                        #sylvia ::cw_std::from_slice::<ContractExecMsg>(&msg)?
                            .dispatch(self, (deps, env, info))
                            .map_err(Into::into)
                    }

                    fn instantiate(
                        &self,
                        deps: #sylvia ::cw_std::DepsMut<#sylvia ::cw_std::Empty>,
                        env: #sylvia ::cw_std::Env,
                        info: #sylvia ::cw_std::MessageInfo,
                        msg: Vec<u8>,
                    ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                        #sylvia ::cw_std::from_slice::<InstantiateMsg>(&msg)?
                            .dispatch(self, (deps, env, info))
                            .map_err(Into::into)
                    }

                    fn query(
                        &self,
                        deps: #sylvia ::cw_std::Deps<#sylvia ::cw_std::Empty>,
                        env: #sylvia ::cw_std::Env,
                        msg: Vec<u8>,
                    ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Binary> {
                        #sylvia ::cw_std::from_slice::<ContractQueryMsg>(&msg)?
                            .dispatch(self, (deps, env))
                            .map_err(Into::into)
                    }

                    fn sudo(
                        &self,
                        _deps: #sylvia ::cw_std::DepsMut<#sylvia ::cw_std::Empty>,
                        _env: #sylvia ::cw_std::Env,
                        _msg: Vec<u8>,
                    ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                        #sylvia ::anyhow::bail!("sudo not implemented for contract")
                    }

                    fn reply(
                        &self,
                        deps: #sylvia ::cw_std::DepsMut<#sylvia ::cw_std::Empty>,
                        env: #sylvia ::cw_std::Env,
                        msg: #sylvia ::cw_std::Reply,
                    ) -> #sylvia ::anyhow::Result<#sylvia ::cw_std::Response<#custom_msg>> {
                        #reply
                    }

                    fn migrate(
                        &self,
                        deps: #sylvia ::cw_std::DepsMut<#sylvia ::cw_std::Empty>,
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
