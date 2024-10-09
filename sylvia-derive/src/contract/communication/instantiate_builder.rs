use convert_case::Case;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{GenericParam, Type, WhereClause};

use crate::crate_module;
use crate::parser::MsgType;
use crate::types::msg_variant::MsgVariant;
use crate::utils::{get_ident_from_type, SvCasing};

pub struct InstantiateBuilder<'a> {
    contract: Type,
    used_generics: &'a [&'a GenericParam],
    where_clause: &'a Option<WhereClause>,
    instantiate_variant: &'a MsgVariant<'a>,
}

impl<'a> InstantiateBuilder<'a> {
    pub fn new(
        contract: Type,
        used_generics: &'a [&'a syn::GenericParam],
        where_clause: &'a Option<WhereClause>,
        instantiate_variant: &'a MsgVariant<'a>,
    ) -> Self {
        Self {
            contract,
            used_generics,
            where_clause,
            instantiate_variant,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            contract,
            used_generics,
            where_clause,
            instantiate_variant,
        } = self;

        let contract_name = get_ident_from_type(contract);
        let msg_type = MsgType::Instantiate;

        let trait_name = Ident::new(
            &format!("{}InstantiateBuilder", &contract_name.to_string()),
            contract_name.span(),
        );
        let method_name = contract_name.to_case(Case::Snake);
        let fields_names = instantiate_variant.as_fields_names();
        let parameters = instantiate_variant.emit_method_field();
        let msg_name = msg_type.emit_msg_name();

        quote! {
            pub trait #trait_name {
                fn #method_name < #(#used_generics),* > (code_id: u64, #(#parameters),* ) -> #sylvia ::cw_std::StdResult < #sylvia ::builder::instantiate::InstantiateBuilder> #where_clause;
            }

            impl #trait_name for #sylvia ::builder::instantiate::InstantiateBuilder {
                fn #method_name < #(#used_generics),* > (code_id: u64, #(#parameters,)* ) -> #sylvia ::cw_std::StdResult< #sylvia ::builder::instantiate::InstantiateBuilder> #where_clause {
                    let msg = #msg_name ::< #(#used_generics),* > ::new( #(#fields_names),* );
                    let msg = #sylvia ::cw_std::to_json_binary(&msg)?;
                    Ok( #sylvia ::builder::instantiate::InstantiateBuilder::new(msg, code_id))
                }
            }
        }
    }
}
