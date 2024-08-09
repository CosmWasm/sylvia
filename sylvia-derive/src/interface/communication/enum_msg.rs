use crate::parser::attributes::MsgAttrForwarding;
use crate::parser::{Custom, MsgType, ParsedSylviaAttributes};
use crate::types::associated_types::{AssociatedTypes, ItemType, EXEC_TYPE, QUERY_TYPE};
use crate::types::msg_variant::MsgVariants;
use crate::utils::emit_bracketed_generics;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, Ident, ItemTrait, Type};

/// Representation of single enum message
pub struct EnumMessage<'a> {
    source: &'a ItemTrait,
    variants: MsgVariants<'a, Ident>,
    associated_types: &'a AssociatedTypes<'a>,
    msg_ty: MsgType,
    resp_type: Type,
    query_type: Type,
    msg_attrs_to_forward: Vec<MsgAttrForwarding>,
}

impl<'a> EnumMessage<'a> {
    pub fn new(
        source: &'a ItemTrait,
        msg_ty: MsgType,
        custom: &'a Custom,
        variants: MsgVariants<'a, Ident>,
        associated_types: &'a AssociatedTypes<'a>,
    ) -> Self {
        let trait_name = &source.ident;
        let associated_exec =
            associated_types.emit_contract_custom_type_accessor(trait_name, EXEC_TYPE);
        let associated_query =
            associated_types.emit_contract_custom_type_accessor(trait_name, QUERY_TYPE);

        let resp_type = custom
            .msg
            .clone()
            .or(associated_exec)
            .unwrap_or_else(Custom::default_type);

        let query_type = custom
            .query
            .clone()
            .or(associated_query)
            .unwrap_or_else(Custom::default_type);

        let msg_attrs_to_forward = ParsedSylviaAttributes::new(source.attrs.iter())
            .msg_attrs_forward
            .into_iter()
            .filter(|attr| attr.msg_type == msg_ty)
            .collect();

        Self {
            source,
            variants,
            associated_types,
            msg_ty,
            resp_type,
            query_type,
            msg_attrs_to_forward,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            source,
            variants,
            associated_types,
            msg_ty,
            resp_type,
            query_type,
            msg_attrs_to_forward,
        } = self;

        let trait_name = &source.ident;
        let enum_name = msg_ty.emit_msg_name();
        let unique_enum_name =
            Ident::new(&format!("{}{}", trait_name, enum_name), enum_name.span());

        let match_arms = variants.emit_dispatch_legs();
        let mut msgs = variants.as_names_snake_cased();
        msgs.sort();
        let msgs_cnt = msgs.len();
        let variants_constructors = variants.emit_constructors();
        let msg_variants = variants.emit();

        let ctx_type = msg_ty.emit_ctx_type(query_type);
        let dispatch_type = msg_ty.emit_result_type(resp_type, &parse_quote!(ContractT::Error));

        let used_generics = variants.used_generics();
        let unused_generics = variants.unused_generics();
        let where_predicates = associated_types
            .without_error()
            .map(ItemType::as_where_predicate);
        let where_clause = variants.where_clause();
        let contract_predicate = associated_types.emit_contract_predicate(trait_name);

        let phantom_variant = variants.emit_phantom_variant();
        let phatom_match_arm = variants.emit_phantom_match_arm();
        let bracketed_used_generics = emit_bracketed_generics(used_generics);

        let ep_name = msg_ty.emit_ep_name();
        let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), enum_name.span());
        let derive_call = msg_ty.emit_derive_call();
        let msg_attrs_to_forward = msg_attrs_to_forward.iter().map(|attr| &attr.attrs);

        quote! {
            #[allow(clippy::derive_partial_eq_without_eq)]
            #derive_call
            #( #[ #msg_attrs_to_forward ] )*
            #[serde(rename_all="snake_case")]
            pub enum #unique_enum_name #bracketed_used_generics {
                #(#msg_variants,)*
                #phantom_variant
            }
            pub type #enum_name #bracketed_used_generics = #unique_enum_name #bracketed_used_generics;

            impl #bracketed_used_generics #unique_enum_name #bracketed_used_generics #where_clause {
                pub fn dispatch<ContractT, #(#unused_generics,)*>(self, contract: &ContractT, ctx: #ctx_type)
                    -> #dispatch_type
                where
                    #(#where_predicates,)*
                    #contract_predicate
                {
                    use #unique_enum_name::*;

                    match self {
                        #(#match_arms,)*
                        #phatom_match_arm
                    }
                }
                #(#variants_constructors)*
            }

            pub const fn #messages_fn_name () -> [&'static str; #msgs_cnt] {
                [#(#msgs,)*]
            }
        }
    }
}
