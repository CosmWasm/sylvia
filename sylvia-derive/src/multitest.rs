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
use crate::parser::{parse_struct_message, ContractMessageAttr, MsgAttr, MsgType};
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
    source: &'a ItemImpl,
    generics: &'a [&'a GenericParam],
    contract_name: &'a Ident,
    proxy_name: Ident,
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
    ) -> Self {
        let mut is_migrate = false;

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
                        quote! { cw_multi_test::AppResponse }
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
            source,
            generics,
            contract_name,
            proxy_name,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            messages,
            error_type,
            proxy_name,
            source,
            ..
        } = self;
        let sylvia = crate_module();

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
                    pub fn #name (&self, #(#params,)* ) -> #sylvia ::multitest::ExecProxy::<#error_type, ExecMsg> {
                        let msg = ExecMsg:: #name ( #(#arguments),* );

                        #sylvia ::multitest::ExecProxy::new(&self.contract_addr, msg, &self.app)
                    }
                }
            } else {
                quote! {
                    pub fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                        let msg = QueryMsg:: #name ( #(#arguments),* );

                        self.app
                            .app
                            .borrow()
                            .wrap()
                            .query_wasm_smart(self.contract_addr.clone(), &msg)
                            .map_err(Into::into)
                    }
                }
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
                        let module = &module.segments[0].ident;
                        let method_name = Ident::new(&format!("{}_proxy", module), module.span());
                        let proxy_name = Ident::new(
                            &format!("{}Proxy", module.to_string().to_case(Case::UpperCamel)),
                            module.span(),
                        );

                        quote! {
                            pub fn #method_name (&self) -> #proxy_name <'app> {
                                #proxy_name ::new(self.contract_addr.clone(), self.app)
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

        quote! {
            #[cfg(test)]
            pub mod multitest_utils {
                use super::*;
                use cw_multi_test::Executor;

                pub struct #proxy_name <'app> {
                    pub contract_addr: cosmwasm_std::Addr,
                    pub app: &'app #sylvia ::multitest::App,
                }

                impl<'app> #proxy_name <'app> {
                    pub fn new(contract_addr: cosmwasm_std::Addr, app: &'app #sylvia ::multitest::App) -> Self {
                        #proxy_name{ contract_addr, app }
                    }

                    #(#messages)*

                    #(#interfaces)*
                }

                impl<'app> From<(cosmwasm_std::Addr, &'app #sylvia ::multitest::App)> for #proxy_name<'app> {
                    fn from(input: (cosmwasm_std::Addr, &'app #sylvia ::multitest::App)) -> #proxy_name<'app> {
                        #proxy_name::new(input.0, input.1)
                    }
                }

                #contract_block
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

        let code_id = Ident::new(&format!("{}CodeId", contract_name), contract_name.span());

        quote! {
            #impl_contract

            pub struct #code_id <'app> {
                code_id: u64,
                app: &'app #sylvia ::multitest::App,
            }

            impl<'app> #code_id <'app> {
                pub fn store_code(app: &'app mut #sylvia ::multitest::App) -> Self {
                    let code_id = app
                        .app
                        .borrow_mut()
                        .store_code(Box::new(#contract_name ::new()));
                    Self { code_id, app }
                }

                pub fn code_id(&self) -> u64 {
                    self.code_id
                }

                pub fn instantiate(
                    &self, #(#fields,)*
                ) -> InstantiateProxy<'_, 'app> {
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

            pub struct InstantiateProxy<'a, 'app> {
                code_id: &'a #code_id <'app>,
                funds: &'a [cosmwasm_std::Coin],
                label: &'a str,
                admin: Option<String>,
                msg: InstantiateMsg,
            }

            impl<'a, 'app> InstantiateProxy<'a, 'app> {
                pub fn with_funds(self, funds: &'a [cosmwasm_std::Coin]) -> Self {
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
                pub fn call(self, sender: &str) -> Result<#proxy_name<'app>, #error_type> {
                    self.code_id
                        .app
                        .app
                        .borrow_mut()
                        .instantiate_contract(
                            self.code_id.code_id,
                            cosmwasm_std::Addr::unchecked(sender),
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

    fn generate_impl_contract(&self) -> TokenStream {
        let contract = &self.contract;

        // MigrateMsg is not generated all the time in contrary to Exec, Query and Instantiate.
        let migrate_body = if self.is_migrate {
            quote! {
                cosmwasm_std::from_slice::<MigrateMsg>(&msg)?
                    .dispatch(self, (deps, env))
                    .map_err(Into::into)
            }
        } else {
            quote! {
                anyhow::bail!("migrate not implemented for contract")
            }
        };
        quote! {
            impl cw_multi_test::Contract<cosmwasm_std::Empty> for #contract {
                fn execute(
                    &self,
                    deps: cosmwasm_std::DepsMut<cosmwasm_std::Empty>,
                    env: cosmwasm_std::Env,
                    info: cosmwasm_std::MessageInfo,
                    msg: Vec<u8>,
                ) -> anyhow::Result<cosmwasm_std::Response<cosmwasm_std::Empty>> {
                    cosmwasm_std::from_slice::<ContractExecMsg>(&msg)?
                        .dispatch(self, (deps, env, info))
                        .map_err(Into::into)
                }

                fn instantiate(
                    &self,
                    deps: cosmwasm_std::DepsMut<cosmwasm_std::Empty>,
                    env: cosmwasm_std::Env,
                    info: cosmwasm_std::MessageInfo,
                    msg: Vec<u8>,
                ) -> anyhow::Result<cosmwasm_std::Response<cosmwasm_std::Empty>> {
                    cosmwasm_std::from_slice::<InstantiateMsg>(&msg)?
                        .dispatch(self, (deps, env, info))
                        .map_err(Into::into)
                }

                fn query(
                    &self,
                    deps: cosmwasm_std::Deps<cosmwasm_std::Empty>,
                    env: cosmwasm_std::Env,
                    msg: Vec<u8>,
                ) -> anyhow::Result<cosmwasm_std::Binary> {
                    cosmwasm_std::from_slice::<ContractQueryMsg>(&msg)?
                        .dispatch(self, (deps, env))
                        .map_err(Into::into)
                }

                fn sudo(
                    &self,
                    _deps: cosmwasm_std::DepsMut<cosmwasm_std::Empty>,
                    _env: cosmwasm_std::Env,
                    _msg: Vec<u8>,
                ) -> anyhow::Result<cosmwasm_std::Response<cosmwasm_std::Empty>> {
                    anyhow::bail!("sudo not implemented for contract")
                }

                fn reply(
                    &self,
                    _deps: cosmwasm_std::DepsMut<cosmwasm_std::Empty>,
                    _env: cosmwasm_std::Env,
                    _msg: cosmwasm_std::Reply,
                ) -> anyhow::Result<cosmwasm_std::Response<cosmwasm_std::Empty>> {
                    anyhow::bail!("reply not implemented for contract")
                }

                fn migrate(
                    &self,
                    deps: cosmwasm_std::DepsMut<cosmwasm_std::Empty>,
                    env: cosmwasm_std::Env,
                    msg: Vec<u8>,
                ) -> anyhow::Result<cosmwasm_std::Response<cosmwasm_std::Empty>> {
                    #migrate_body
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

        quote! {
            #[cfg(test)]
            pub mod trait_utils {
                pub struct #proxy_name <'app> {
                    pub contract_addr: #sylvia ::cw_std::Addr,
                    pub app: &'app #sylvia ::multitest::App,
                }
                impl<'app> #proxy_name <'app> {
                    pub fn new(contract_addr: #sylvia ::cw_std::Addr, app: &'app #sylvia ::multitest::App) -> Self {
                        #proxy_name { contract_addr, app }
                    }
                }
                impl Into<#sylvia ::cw_std::Addr> for #proxy_name <'_> {
                    fn into(self) -> #sylvia ::cw_std::Addr {
                        self.contract_addr
                    }
                }
            }
        }
    }
}
