use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{GenericParam, ItemImpl};

use crate::associated_types::AssociatedTypes;
use crate::crate_module;
use crate::interfaces::Interfaces;

pub struct InterfaceRemote<'a> {
    associated_types: &'a AssociatedTypes<'a>,
}

pub struct ContractRemote<'a> {
    interfaces: &'a Interfaces,
    contract_generics: Vec<&'a GenericParam>,
}

impl<'a> InterfaceRemote<'a> {
    pub fn new(associated_types: &'a AssociatedTypes<'a>) -> Self {
        Self { associated_types }
    }

    pub fn emit(&self) -> TokenStream {
        let generics: Vec<_> = self.associated_types.as_names().collect();

        emit(&generics)
    }
}

impl<'a> ContractRemote<'a> {
    pub fn new(source: &'a ItemImpl, interfaces: &'a Interfaces) -> Self {
        let contract_generics = source.generics.params.iter().collect();
        Self {
            contract_generics,
            interfaces,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let from_impls = self
            .interfaces
            .emit_remote_from_impl(&self.contract_generics);
        let querier = emit(&self.contract_generics);

        quote! {
            #querier
            #(#from_impls)*
        }
    }
}

fn emit(generics: &[impl ToTokens]) -> TokenStream {
    let sylvia = crate_module();

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

    }
}
