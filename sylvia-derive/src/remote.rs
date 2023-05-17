use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::ItemImpl;

use crate::crate_module;
use crate::parser::ContractMessageAttr;

pub struct Remote {
    interfaces: Vec<ContractMessageAttr>,
}

impl Remote {
    pub fn for_contract(source: &ItemImpl) -> Self {
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

    pub fn for_interface() -> Self {
        Self { interfaces: vec![] }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let from_implementations = self.interfaces.iter().map(|interface| {
            let ContractMessageAttr { module, .. } = interface;

            quote! {
                impl<'a> From<&'a Remote<'a>> for #module ::Remote<'a> {
                    fn from(remote: &'a Remote) -> Self {
                        Self (
                            std::borrow::Cow::Borrowed(&remote.0)
                        )
                    }
                }
            }
        });

        quote! {
            pub struct Remote<'a>(pub std::borrow::Cow<'a, #sylvia ::cw_std::Addr>);

            impl Remote<'static> {
                pub fn new(addr: #sylvia ::cw_std::Addr) -> Self {
                    Self(std::borrow::Cow::Owned(addr))
                }
            }

            impl<'a> Remote<'a> {
                pub fn borrowed(addr: &'a #sylvia ::cw_std::Addr) -> Self {
                    Self(std::borrow::Cow::Borrowed(addr))
                }
            }

            #(#from_implementations)*
        }
    }
}
