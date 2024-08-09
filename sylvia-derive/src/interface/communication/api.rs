use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::MsgType;
use crate::types::associated_types::{AssociatedTypes, ItemType};
use crate::types::msg_variant::MsgVariants;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemTrait;

/// Emits `InterfaceMessagesApi` trait.
pub struct Api<'a> {
    source: &'a ItemTrait,
    associated_types: &'a AssociatedTypes<'a>,
}

impl<'a> Api<'a> {
    pub fn new(source: &'a ItemTrait, associated_types: &'a AssociatedTypes<'a>) -> Self {
        Self {
            source,
            associated_types,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            source,
            associated_types,
        } = self;

        let interface_name = &source.ident;
        let generics: Vec<_> = associated_types
            .without_error()
            .map(ItemType::as_name)
            .collect();
        let exec_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Exec,
            &generics,
            &source.generics.where_clause,
        );
        let query_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Query,
            &generics,
            &source.generics.where_clause,
        );
        let sudo_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Sudo,
            &generics,
            &source.generics.where_clause,
        );

        let exec_generics = &exec_variants.used_generics();
        let query_generics = &query_variants.used_generics();
        let sudo_generics = &sudo_variants.used_generics();

        quote! {
            pub trait InterfaceMessagesApi {
                type Exec;
                type Query;
                type Sudo;
            }

            impl<Contract: #interface_name> InterfaceMessagesApi for Contract {
                type Exec = ExecMsg < #(<Contract as #interface_name >:: #exec_generics,)* >;
                type Query = QueryMsg < #(<Contract as #interface_name >:: #query_generics,)* >;
                type Sudo = SudoMsg < #(<Contract as #interface_name >:: #sudo_generics ,)* >;
            }

            impl<'sv_iface_msg_api, Error, #(#generics),*> InterfaceMessagesApi for dyn #interface_name < Error = Error, #(#generics = #generics,)* > + 'sv_iface_msg_api {
                type Exec = ExecMsg < #(#exec_generics,)* >;
                type Query = QueryMsg < #(#query_generics,)* >;
                type Sudo = SudoMsg < #(#sudo_generics,)* >;
            }
        }
    }
}
