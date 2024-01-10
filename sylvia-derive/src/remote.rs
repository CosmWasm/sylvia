use proc_macro2::TokenStream;
use quote::quote;

use crate::associated_types::AssociatedTypes;
use crate::crate_module;
use crate::interfaces::Interfaces;
use crate::parser::ContractMessageAttr;

pub struct Remote<'a> {
    interfaces: &'a Interfaces,
    associated_types: &'a AssociatedTypes<'a>,
}

impl<'a> Remote<'a> {
    pub fn new(interfaces: &'a Interfaces, associated_types: &'a AssociatedTypes<'a>) -> Self {
        Self {
            interfaces,
            associated_types,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let generics = self.associated_types.as_names();

        let from_implementations = self.interfaces.interfaces().iter().map(|interface| {
            let ContractMessageAttr {
                module, generics, ..
            } = interface;

            quote! {
                impl<'a> From<&'a Remote<'a>> for #module ::sv::Remote<'a, #generics > {
                    fn from(remote: &'a Remote) -> Self {
                        #module ::sv::Remote::borrowed(remote.as_ref())
                    }
                }
            }
        });

        quote! {
            #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
            pub struct Remote<'a, #(#generics,)* > {
                addr: std::borrow::Cow<'a, #sylvia ::cw_std::Addr>,
                #[serde(skip)]
                _phantom: std::marker::PhantomData<( #(#generics,)* )>,
            }

            impl<'a, #(#generics,)* > Remote<'a, #(#generics,)* > {
                pub fn new(addr: #sylvia ::cw_std::Addr) -> Self {
                    Self {
                        addr: std::borrow::Cow::Owned(addr),
                        _phantom: std::marker::PhantomData
                    }
                }

                pub fn borrowed(addr: &'a #sylvia ::cw_std::Addr) -> Self {
                    Self {
                        addr: std::borrow::Cow::Borrowed(addr),
                        _phantom: std::marker::PhantomData}
                    }

                pub fn querier<C: #sylvia ::cw_std::CustomQuery>(&'a self, querier: &'a #sylvia ::cw_std::QuerierWrapper<'a, C>) -> BoundQuerier<'a, C, #(#generics,)* > {
                    BoundQuerier {
                        contract: &self.addr,
                        querier,
                        _phantom: std::marker::PhantomData,
                    }
                }
            }

            impl<'a, #(#generics,)* > AsRef<#sylvia ::cw_std::Addr> for Remote<'a, #(#generics,)* > {
                fn as_ref(&self) -> &#sylvia ::cw_std::Addr {
                    &self.addr
                }
            }

            #(#from_implementations)*
        }
    }
}
