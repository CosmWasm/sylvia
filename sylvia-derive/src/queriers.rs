use proc_macro2::TokenStream;
use quote::quote;

use crate::crate_module;

pub struct Querier;

impl Querier {
    pub fn new() -> Self {
        Self
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        quote! {
            pub struct Querier {

            }
        }
    }
}
