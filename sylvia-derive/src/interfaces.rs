use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::ItemImpl;

use crate::crate_module;
use crate::parser::ContractMessageAttr;

pub struct Interfaces {
    interfaces: Vec<ContractMessageAttr>,
}

impl Interfaces {
    pub fn new(source: &ItemImpl) -> Self {
        let interfaces: Vec<_> = source
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("messages"))
            .filter_map(|attr| {
                let interface = match ContractMessageAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(interface) => interface,
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        return None;
                    }
                };

                Some(interface)
            })
            .collect();

        Self { interfaces }
    }

    pub fn interfaces(&self) -> &[ContractMessageAttr] {
        &self.interfaces
    }

    pub fn emit_querier_from_impl(&self) -> Vec<TokenStream> {
        let sylvia = crate_module();

        self.interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, .. } = interface;
                quote! {
                    impl<'a, C: #sylvia ::cw_std::CustomQuery> From<&'a BoundQuerier<'a, C>> for #module ::BoundQuerier<'a, C> {
                        fn from(querier: &'a BoundQuerier<'a, C>) -> Self {
                            Self::borrowed(querier.contract(),  querier.querier())
                        }
                    }
                }
            })
            .collect()
    }
}
