use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, ItemImpl, Pat, PatType, Type};

use crate::crate_module;
use crate::parser::{ContractMessageAttr, MsgAttr, MsgType};
use crate::utils::extract_return_type;

struct MessageSignature<'a> {
    pub name: &'a Ident,
    pub params: Vec<&'a FnArg>,
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
}

impl<'a> MultitestHelpers<'a> {
    pub fn new(source: &'a ItemImpl, is_trait: bool, contract_error: &'a Type) -> Self {
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
                    let params: Vec<_> = sig.inputs.iter().skip(2).collect();
                    let arguments: Vec<_> = params
                        .iter()
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

        // let interfaces: Vec<_> = source
        //     .attrs
        //     .iter()
        //     .filter(|attr| attr.path.is_ident("messages"))
        //     .filter_map(|attr| {
        //         let interface = match ContractMessageAttr::parse.parse2(attr.tokens.clone()) {
        //             Ok(interface) => interface,
        //             Err(err) => {
        //                 emit_error!(attr.span(), err);
        //                 return None;
        //             }
        //         };
        //
        //         Some(interface)
        //     })
        //     .collect();

        let error_type = if is_trait {
            let error_type: Vec<_> = source
                .items
                .iter()
                .filter_map(|item| match item {
                    ImplItem::Type(ty) => {
                        if ty.ident != "Error" {
                            return None;
                        }

                        let ty = &ty.ty;
                        let segments = match ty {
                            Type::Path(path) => &path.path.segments,
                            _ => unreachable!(),
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

        Self {
            messages,
            error_type,
            contract: &source.self_ty,
            is_trait,
            is_migrate,
        }
    }
    pub fn emit(&self) -> TokenStream {
        let Self {
            messages,
            error_type,
            contract,
            is_trait,
            is_migrate,
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

        let impl_contract = self.generate_impl_contract();

        quote! {
            #[cfg(test)]
            pub mod multitest_utils {
                use super::*;
                use cw_multi_test::Executor;

                pub struct Proxy<'app> {
                    pub contract_addr: cosmwasm_std::Addr,
                    pub app: &'app #sylvia ::multitest::App,
                }

                impl<'app> Proxy<'app> {
                    pub fn new(contract_addr: cosmwasm_std::Addr, app: &'app #sylvia ::multitest::App) -> Self {
                        Proxy{ contract_addr, app }
                    }

                    #(#messages)*

                }

                impl<'app> From<(cosmwasm_std::Addr, &'app #sylvia ::multitest::App)> for Proxy<'app> {
                    fn from(input: (cosmwasm_std::Addr, &'app #sylvia ::multitest::App)) -> Proxy<'app> {
                        Proxy::new(input.0, input.1)
                    }
                }

                #impl_contract
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

        if self.is_trait {
            quote! {}
        } else {
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
}
