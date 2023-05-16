use proc_macro2::TokenStream;
use quote::quote;

use crate::crate_module;

pub struct Remote {}

impl Remote {
    pub fn new() -> Self {
        Self {}
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        quote! {
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
        }
    }
}
