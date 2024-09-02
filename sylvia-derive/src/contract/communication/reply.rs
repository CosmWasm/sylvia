use convert_case::{Case, Casing};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, Ident};

use crate::types::msg_variant::MsgVariants;

pub struct Reply<'a> {
    variants: &'a MsgVariants<'a, GenericParam>,
}

impl<'a> Reply<'a> {
    pub fn new(variants: &'a MsgVariants<'a, GenericParam>) -> Self {
        Self { variants }
    }

    pub fn emit(&self) -> TokenStream {
        let unique_handlers = self.emit_reply_ids();

        quote! {
            #(#unique_handlers)*
        }
    }

    fn emit_reply_ids(&self) -> impl Iterator<Item = TokenStream> + 'a {
        self.variants
            .as_reply_ids()
            .enumerate()
            .map(|(id, reply_id)| {
                let id = id as u64;

                quote! {
                    pub const #reply_id : u64 = #id ;
                }
            })
    }
}

trait ReplyVariants<'a> {
    fn as_reply_ids(&'a self) -> impl Iterator<Item = Ident> + 'a;
}

impl<'a> ReplyVariants<'a> for MsgVariants<'a, GenericParam> {
    fn as_reply_ids(&'a self) -> impl Iterator<Item = Ident> + 'a {
        self.variants()
            .flat_map(|variant| {
                if variant.msg_attr().handlers().is_empty() {
                    return vec![variant.function_name()];
                }
                variant.msg_attr().handlers().iter().collect()
            })
            .unique()
            .map(|handler| {
                let reply_id =
                    format! {"{}_REPLY_ID", handler.to_string().to_case(Case::UpperSnake)};
                Ident::new(&reply_id, handler.span())
            })
    }
}
