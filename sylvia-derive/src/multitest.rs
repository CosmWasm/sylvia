use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    spanned::Spanned,
    FnArg, GenericParam, ItemTrait, Pat, PatType, TraitItem, WhereClause, WherePredicate,
};

use crate::{
    check_generics::CheckGenerics,
    crate_module,
    parser::{InterfaceArgs, MsgAttr, MsgType},
    utils::filter_wheres,
};

struct MessageSignature<'a> {
    pub name: &'a Ident,
    pub params: Vec<&'a FnArg>,
    pub arguments: Vec<&'a Ident>,
}

pub struct MultitestHelpers<'a> {
    trait_name: &'a Ident,
    messages: Vec<MessageSignature<'a>>,
    generics: Vec<&'a GenericParam>,
    unused_generics: Vec<&'a GenericParam>,
    all_generics: &'a [&'a GenericParam],
    wheres: Vec<&'a WherePredicate>,
    full_where: Option<&'a WhereClause>,
    msg_ty: MsgType,
    args: &'a InterfaceArgs,
}

impl<'a> MultitestHelpers<'a> {
    pub fn new(
        source: &'a ItemTrait,
        ty: MsgType,
        generics: &'a [&'a GenericParam],
        args: &'a InterfaceArgs,
    ) -> Self {
        let trait_name = &source.ident;

        let generics_checker = CheckGenerics::new(generics);
        let messages: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Method(method) => {
                    // We want to generate helepers only for contract messages
                    method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;

                    let sig = &method.sig;
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
                                // println!("ident is {:#?}", ident.ident);
                                Some(&ident.ident)
                            }
                            _ => None,
                        })
                        .collect();

                    Some(MessageSignature {
                        name,
                        params,
                        arguments,
                    })
                }
                _ => None,
            })
            .collect();

        let (used_generics, unused_generics) = generics_checker.used_unused();
        let wheres = filter_wheres(&source.generics.where_clause, generics, &used_generics);

        Self {
            trait_name,
            messages,
            generics: used_generics,
            unused_generics,
            all_generics: generics,
            wheres,
            full_where: source.generics.where_clause.as_ref(),
            msg_ty: ty,
            args,
        }
    }
    pub fn emit(&self) -> TokenStream {
        let Self {
            trait_name,
            messages,
            generics,
            unused_generics,
            all_generics,
            wheres,
            full_where,
            msg_ty,
            args,
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
            } = msg;
            quote! {
                pub fn #name (&self, params: #sylvia ::multitest::ExecParams, #(#params,)* ) {

                }
            }
        });

        quote! {
            #[cfg(test)]
            mod test_utils {
                use super::*;

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
