use proc_macro2::TokenStream;
use quote::quote;

use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::parser::ContractMessageAttr;

pub struct Remote<'a> {
    interfaces: &'a Interfaces,
}

impl<'a> Remote<'a> {
    pub fn new(interfaces: &'a Interfaces) -> Self {
        Self { interfaces }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let from_implementations = self.interfaces.interfaces().iter().map(|interface| {
            let ContractMessageAttr { module, .. } = interface;

            quote! {
                impl<'a> From<&'a Remote<'a>> for #module ::sv::Remote<'a> {
                    fn from(remote: &'a Remote) -> Self {
                        #module ::sv::Remote::borrowed(remote.as_ref())
                    }
                }
            }
        });

        quote! {
            #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
            pub struct Remote<'a>(std::borrow::Cow<'a, #sylvia ::cw_std::Addr>);

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

            impl<'a> AsRef<#sylvia ::cw_std::Addr> for Remote<'a> {
                fn as_ref(&self) -> &#sylvia ::cw_std::Addr {
                    &self.0
                }
            }

            impl Remote<'_> {
                pub fn querier<'a, C: #sylvia ::cw_std::CustomQuery>(&'a self, querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>) -> BoundQuerier<'a, C> {
                    BoundQuerier {
                        contract: &self.0,
                        querier,
                    }
                }
            }

            #(#from_implementations)*
        }
    }
}
