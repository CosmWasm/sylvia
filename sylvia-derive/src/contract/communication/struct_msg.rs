use crate::crate_module;
use crate::parser::attributes::MsgAttrForwarding;
use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::{ContractErrorAttr, Custom, MsgType, ParsedSylviaAttributes};
use crate::types::msg_field::MsgField;
use crate::types::msg_variant::MsgVariants;
use crate::utils::{as_where_clause, emit_bracketed_generics, filter_wheres};
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::spanned::Spanned;
use syn::{GenericParam, ItemImpl, Type};

/// Representation of single struct message
pub struct StructMessage<'a> {
    source: &'a ItemImpl,
    contract_type: &'a Type,
    variants: MsgVariants<'a, GenericParam>,
    generics: &'a [&'a GenericParam],
    error: &'a ContractErrorAttr,
    custom: &'a Custom,
    msg_attrs_to_forward: Vec<MsgAttrForwarding>,
}

impl<'a> StructMessage<'a> {
    pub fn new(
        source: &'a ItemImpl,
        msg_ty: MsgType,
        generics: &'a [&'a GenericParam],
        error: &'a ContractErrorAttr,
        custom: &'a Custom,
    ) -> Option<StructMessage<'a>> {
        let contract_type = &source.self_ty;

        let variants = MsgVariants::new(
            source.as_variants(),
            msg_ty,
            generics,
            &source.generics.where_clause,
        );

        if variants.variants().count() == 0 && variants.msg_ty() == MsgType::Instantiate {
            emit_error!(
                source.span(), "Missing instantiation message.";
                note = source.span() => "`sylvia::contract` requires exactly one method marked with `#[sv::msg(instantiation)]` attribute."
            );
            return None;
        } else if variants.variants().count() > 1 {
            let mut variants = variants.variants();
            let first_method = variants.next().map(|v| v.function_name());
            let obsolete = variants.next().map(|v| v.function_name());
            emit_error!(
                first_method.span(), "More than one instantiation or migration message";
                note = obsolete.span() => "Instantiation/Migration message previously defined here"
            );
            return None;
        }

        let msg_attrs_to_forward = ParsedSylviaAttributes::new(source.attrs.iter())
            .msg_attrs_forward
            .into_iter()
            .filter(|attr| attr.msg_type == msg_ty)
            .collect();

        Some(Self {
            source,
            contract_type,
            variants,
            generics,
            error,
            custom,
            msg_attrs_to_forward,
        })
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            source,
            contract_type,
            variants,
            generics,
            error,
            custom,
            msg_attrs_to_forward,
        } = self;

        let Some(variant) = variants.get_only_variant() else {
            return quote! {};
        };

        let used_generics = variants.used_generics();
        let unused_generics = variants.unused_generics();
        let full_where = &source.generics.where_clause;
        let wheres = filter_wheres(full_where, generics, used_generics);
        let where_clause = as_where_clause(&wheres);
        let bracketed_used_generics = emit_bracketed_generics(used_generics);
        let bracketed_unused_generics = emit_bracketed_generics(unused_generics);

        let ret_type = variant
            .msg_type()
            .emit_result_type(&custom.msg_or_default(), &error.error);
        let name = variant.msg_type().emit_msg_name();
        let function_name = variant.function_name();
        let mut msg_name = variant.msg_type().emit_msg_name();
        msg_name.set_span(function_name.span());

        let ctx_type = variant.msg_type().emit_ctx_type(&custom.query_or_default());
        let fields_names: Vec<_> = variant.fields().iter().map(MsgField::name).collect();
        let parameters = variant.fields().iter().map(MsgField::emit_method_field);
        let fields = variant.fields().iter().map(MsgField::emit_pub);

        let msg_attrs_to_forward = msg_attrs_to_forward.iter().map(|attr| &attr.attrs);

        quote! {
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
            #( #[ #msg_attrs_to_forward ] )*
            #[serde(rename_all="snake_case")]
            pub struct #name #bracketed_used_generics {
                #(#fields,)*
            }

            impl #bracketed_used_generics #name #bracketed_used_generics #where_clause {
                pub fn new(#(#parameters,)*) -> Self {
                    Self { #(#fields_names,)* }
                }

                pub fn dispatch #bracketed_unused_generics (self, contract: &#contract_type, ctx: #ctx_type) -> #ret_type #full_where
                {
                    let Self { #(#fields_names,)* } = self;
                    contract.#function_name(Into::into(ctx), #(#fields_names,)*).map_err(Into::into)
                }
            }
        }
    }
}
