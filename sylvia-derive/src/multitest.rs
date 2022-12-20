use convert_case::{Case, Casing};
use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::{FnArg, GenericParam, ImplItem, ItemImpl, Pat, PatType, Type};

use crate::crate_module;
use crate::parser::{ContractArgs, MsgAttr, MsgType};
use crate::utils::extract_return_type;

struct MessageSignature<'a> {
    pub name: &'a Ident,
    pub params: Vec<&'a FnArg>,
    pub arguments: Vec<&'a Ident>,
    pub msg_ty: MsgType,
    pub return_type: TokenStream,
}

pub struct MultitestHelpers<'a> {
    trait_name: &'a Ident,
    messages: Vec<MessageSignature<'a>>,
    _args: &'a ContractArgs,
    error_type: &'a Ident,
}

fn extract_trait_name<'a>(source: &'a ItemImpl) -> &'a Ident {
    let Some((_, path, _)) = source.trait_.as_ref() else {
        unreachable!()
    };
    let Some(ident) = path.get_ident() else {
        unreachable!()
    };
    ident
}

impl<'a> MultitestHelpers<'a> {
    pub fn new(
        source: &'a ItemImpl,
        _generics: &'a [&'a GenericParam],
        args: &'a ContractArgs,
    ) -> Self {
        let trait_name = extract_trait_name(source);

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
                    // assert_eq!(segments.len(), 1);

                    Some(&segments[0].ident)
                }
                _ => None,
            })
            .collect();
        println!("error_type = {:#?}", error_type);
        // assert_eq!(error_type.len(), 1);
        let error_type = error_type[0];

        Self {
            trait_name,
            messages,
            _args: args,
            error_type,
        }
    }
    pub fn emit(&self) -> TokenStream {
        let Self {
            trait_name,
            messages,
            _args,
            error_type,
        } = self;
        let sylvia = crate_module();
        let proxy_name = Ident::new(
            &format!("{}Proxy", trait_name.to_string()),
            trait_name.span(),
        );

        let messages = messages.iter().map(|msg| {
            let MessageSignature {
                name,
                params,
                arguments,
                msg_ty,
                return_type,
            } = msg;
            let variant = Ident::new(
                &name.to_string().to_case(Case::UpperCamel),
                name.span(),
            );
            if msg_ty == &MsgType::Exec {
                quote! {
                    pub fn #name (&self, params: #sylvia ::multitest::ExecParams, #(#params,)* ) -> Result<#return_type, #error_type> {
                        let msg = ExecMsg:: #variant { #(#arguments,)* }; 

                        self.app
                            .app
                            .borrow_mut()
                            .execute_contract(
                                params.sender.clone(),
                                self.contract_addr.clone(),
                                &msg,
                                params.funds,
                            )
                            .map_err(|err| err.downcast().unwrap())
                    }
                }
            } else {
                quote! {
                    pub fn #name (&self, #(#params,)* ) -> Result<#return_type, #error_type> {
                        let msg = QueryMsg:: #variant { #(#arguments,)* };

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

        quote! {
            #[cfg(test)]
            mod multitest_utils {
                use super::*;
                use cw_multi_test::Executor;

                pub struct #proxy_name<'app> {
                    pub contract_addr: cosmwasm_std::Addr,
                    pub app: &'app #sylvia ::multitest::App,
                }

                impl<'app> #proxy_name<'app> {
                    pub fn new(contract_addr: cosmwasm_std::Addr, app: &'app #sylvia ::multitest::App) -> Self {
                        #proxy_name{ contract_addr, app }
                    }

                    #(#messages)*

                }

                impl Into<cosmwasm_std::Addr> for #proxy_name<'_> {
                    fn into(self) -> cosmwasm_std::Addr {
                        self.contract_addr
                    }
                }
            }
        }
    }
}
