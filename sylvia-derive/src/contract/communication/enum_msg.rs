use crate::crate_module;
use crate::parser::attributes::MsgAttrForwarding;
use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::{ContractErrorAttr, Custom, MsgType, ParsedSylviaAttributes};
use crate::types::msg_variant::MsgVariants;
use crate::utils::emit_bracketed_generics;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{GenericParam, Ident, ItemImpl, Type, WhereClause};

/// Representation of single enum message
pub struct EnumMessage<'a> {
    variants: MsgVariants<'a, GenericParam>,
    msg_ty: MsgType,
    contract: &'a Type,
    error: &'a ContractErrorAttr,
    custom: &'a Custom,
    where_clause: &'a Option<WhereClause>,
    msg_attrs_to_forward: Vec<MsgAttrForwarding>,
}

impl<'a> EnumMessage<'a> {
    pub fn new(
        source: &'a ItemImpl,
        msg_ty: MsgType,
        generics: &'a [&'a GenericParam],
        error: &'a ContractErrorAttr,
        custom: &'a Custom,
    ) -> Self {
        let where_clause = &source.generics.where_clause;
        let variants = MsgVariants::new(source.as_variants(), msg_ty, generics, where_clause);
        let msg_attrs_to_forward = ParsedSylviaAttributes::new(source.attrs.iter())
            .msg_attrs_forward
            .into_iter()
            .filter(|attr| attr.msg_type == msg_ty)
            .collect();

        Self {
            variants,
            msg_ty,
            contract: &source.self_ty,
            error,
            custom,
            where_clause,
            msg_attrs_to_forward,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            variants,
            msg_ty,
            contract,
            error,
            custom,
            where_clause,
            msg_attrs_to_forward,
            ..
        } = self;

        let enum_name = msg_ty.emit_msg_name();
        let match_arms = variants.emit_dispatch_legs();
        let unused_generics = variants.unused_generics();
        let bracketed_unused_generics = emit_bracketed_generics(unused_generics);
        let used_generics = variants.used_generics();
        let bracketed_used_generics = emit_bracketed_generics(used_generics);

        let mut variant_names = variants.as_names_snake_cased();
        variant_names.sort();
        let variants_cnt = variant_names.len();
        let variants_constructors = variants.emit_constructors();
        let variants = variants.emit();

        let ctx_type = msg_ty.emit_ctx_type(&custom.query_or_default());
        let ret_type = msg_ty.emit_result_type(&custom.msg_or_default(), &error.error);

        let derive_query = match msg_ty {
            MsgType::Query => quote! { #sylvia ::cw_schema::QueryResponses },
            _ => quote! {},
        };

        let ep_name = msg_ty.emit_ep_name();
        let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), contract.span());

        let phantom_variant = msg_ty.emit_phantom_variant(used_generics);
        let phantom_match_arm = match !used_generics.is_empty() {
            true => quote! {
                _Phantom(_) => Err(#sylvia ::cw_std::StdError::generic_err("Phantom message should not be constructed.")).map_err(Into::into),
            },
            false => quote! {},
        };
        let msg_attrs_to_forward = msg_attrs_to_forward.iter().map(|attr| &attr.attrs);

        quote! {
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, #derive_query )]
            #( #[ #msg_attrs_to_forward ] )*
            #[serde(rename_all="snake_case")]
            pub enum #enum_name #bracketed_used_generics {
                #(#variants,)*
                #phantom_variant
            }

            impl #bracketed_used_generics #enum_name #bracketed_used_generics {
                pub fn dispatch #bracketed_unused_generics (self, contract: &#contract, ctx: #ctx_type) -> #ret_type #where_clause {
                    use #enum_name::*;

                    match self {
                        #(#match_arms,)*
                        #phantom_match_arm
                    }
                }

                #(#variants_constructors)*
            }

            pub const fn #messages_fn_name () -> [&'static str; #variants_cnt] {
                [#(#variant_names,)*]
            }
        }
    }
}
