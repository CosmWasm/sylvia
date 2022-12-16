use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::{
    parse::{Parse, Parser},
    spanned::Spanned,
    GenericParam, ItemTrait, TraitItem, WhereClause, WherePredicate,
};

use crate::{
    check_generics::CheckGenerics,
    crate_module,
    message::MsgVariant,
    parser::{InterfaceArgs, MsgAttr, MsgType},
    utils::filter_wheres,
};

pub struct MultitestHelpers<'a> {
    trait_name: &'a Ident,
    variants: Vec<MsgVariant<'a>>,
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

        let mut generics_checker = CheckGenerics::new(generics);
        let variants: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Method(method) => {
                    let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                    let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                        Ok(attr) => attr,
                        Err(err) => {
                            emit_error!(method.span(), err);
                            return None;
                        }
                    };

                    if attr == ty {
                        Some(MsgVariant::new(
                            &method.sig,
                            &mut generics_checker,
                            &trait_name, // placeholder
                            attr,
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        let (used_generics, unused_generics) = generics_checker.used_unused();
        let wheres = filter_wheres(&source.generics.where_clause, generics, &used_generics);

        Self {
            trait_name,
            variants,
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
            variants,
            generics,
            unused_generics,
            all_generics,
            wheres,
            full_where,
            msg_ty,
            args,
        } = self;

        if cfg!(not(feature = "mt")) {
            return quote! {};
        }
        let sylvia = crate_module();
        let proxy_name = Ident::new(
            &format!("{}Proxy", trait_name.to_string()),
            trait_name.span(),
        );

        quote! {
            #[cfg(test)]
            mod test_utils {
                pub struct #proxy_name<'app> {
                    pub contract_addr: cosmwasm_std::Addr,
                    pub app: &'app #sylvia ::multitest::App,
                }

                impl<'app> #proxy_name<'app> {
                    pub fn new(contract_addr: cosmwasm_std::Addr, app: &'app #sylvia ::multitest::App) -> Self {
                        #proxy_name{ contract_addr, app }
                    }
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
