use crate::crate_module;
use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::{Custom, MsgType};
use crate::types::msg_variant::MsgVariants;
use crate::utils::emit_bracketed_generics;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, ItemImpl};

pub struct Api<'a> {
    source: &'a ItemImpl,
    exec_variants: MsgVariants<'a, GenericParam>,
    query_variants: MsgVariants<'a, GenericParam>,
    instantiate_variants: MsgVariants<'a, GenericParam>,
    migrate_variants: MsgVariants<'a, GenericParam>,
    sudo_variants: MsgVariants<'a, GenericParam>,
    generics: &'a [&'a GenericParam],
    custom: &'a Custom,
}

impl<'a> Api<'a> {
    pub fn new(source: &'a ItemImpl, generics: &'a [&'a GenericParam], custom: &'a Custom) -> Self {
        let exec_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Exec,
            generics,
            &source.generics.where_clause,
        );

        let query_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Query,
            generics,
            &source.generics.where_clause,
        );

        let instantiate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Instantiate,
            generics,
            &source.generics.where_clause,
        );

        let migrate_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Migrate,
            generics,
            &source.generics.where_clause,
        );

        let sudo_variants = MsgVariants::new(
            source.as_variants(),
            MsgType::Sudo,
            generics,
            &source.generics.where_clause,
        );

        Self {
            source,
            exec_variants,
            query_variants,
            instantiate_variants,
            migrate_variants,
            sudo_variants,
            generics,
            custom,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            exec_variants,
            query_variants,
            instantiate_variants,
            migrate_variants,
            sudo_variants,
            generics,
            custom,
            ..
        } = self;

        let where_clause = &source.generics.where_clause;
        let contract_name = &source.self_ty;
        let exec_generics = &exec_variants.used_generics();
        let query_generics = &query_variants.used_generics();
        let instantiate_generics = &instantiate_variants.used_generics();
        let migrate_generics = &migrate_variants.used_generics();
        let sudo_generics = &sudo_variants.used_generics();

        let bracket_generics = emit_bracketed_generics(generics);
        let exec_bracketed_generics = emit_bracketed_generics(exec_generics);
        let query_bracketed_generics = emit_bracketed_generics(query_generics);
        let sudo_bracketed_generics = emit_bracketed_generics(sudo_generics);
        let instantiate_bracketed_generics = emit_bracketed_generics(instantiate_generics);
        let migrate_bracketed_generics = emit_bracketed_generics(migrate_generics);

        let migrate_type = if migrate_variants.variants().count() != 0 {
            quote! { type Migrate = MigrateMsg #migrate_bracketed_generics; }
        } else {
            quote! { type Migrate = #sylvia ::cw_std::Empty; }
        };
        let custom_msg = custom.msg_or_default();
        let custom_query = custom.query_or_default();

        quote! {
            impl #bracket_generics #sylvia ::types::ContractApi for #contract_name #where_clause {
                type ContractExec = ContractExecMsg #bracket_generics;
                type ContractQuery = ContractQueryMsg #bracket_generics;
                type ContractSudo = ContractSudoMsg #bracket_generics;
                type Exec = ExecMsg #exec_bracketed_generics;
                type Query = QueryMsg #query_bracketed_generics;
                type Sudo = SudoMsg #sudo_bracketed_generics;
                type Instantiate = InstantiateMsg #instantiate_bracketed_generics;
                #migrate_type
                type Remote<'remote> = #sylvia ::types::Remote<'remote, Self >;
                type Querier<'querier> = #sylvia ::types::BoundQuerier<'querier, #custom_query, Self >;
                type CustomMsg = #custom_msg;
                type CustomQuery = #custom_query;
            }
        }
    }
}
