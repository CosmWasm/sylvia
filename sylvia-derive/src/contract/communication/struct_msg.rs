use crate::crate_module;
use crate::parser::attributes::MsgAttrForwarding;
use crate::parser::check_generics::CheckGenerics;
use crate::parser::{Custom, MsgAttr, MsgType, ParsedSylviaAttributes};
use crate::types::msg_field::MsgField;
use crate::utils::{as_where_clause, emit_bracketed_generics, filter_wheres, process_fields};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    GenericParam, Ident, ImplItem, ImplItemFn, ItemImpl, ReturnType, Type, WhereClause,
    WherePredicate,
};

/// Representation of single struct message
pub struct StructMessage<'a> {
    contract_type: &'a Type,
    fields: Vec<MsgField<'a>>,
    function_name: &'a Ident,
    generics: Vec<&'a GenericParam>,
    unused_generics: Vec<&'a GenericParam>,
    wheres: Vec<&'a WherePredicate>,
    full_where: Option<&'a WhereClause>,
    result: &'a ReturnType,
    msg_attr: MsgAttr,
    custom: &'a Custom,
    msg_attrs_to_forward: Vec<MsgAttrForwarding>,
}

impl<'a> StructMessage<'a> {
    /// Creates new struct message of given type from impl block
    pub fn new(
        source: &'a ItemImpl,
        ty: MsgType,
        generics: &'a [&'a GenericParam],
        custom: &'a Custom,
    ) -> Option<StructMessage<'a>> {
        let mut generics_checker = CheckGenerics::new(generics);

        let contract_type = &source.self_ty;

        let parsed = Self::parse_struct_message(source, ty);
        let (method, msg_attr) = parsed?;

        let function_name = &method.sig.ident;
        let fields = process_fields(&method.sig, &mut generics_checker);
        let (used_generics, unused_generics) = generics_checker.used_unused();
        let wheres = filter_wheres(&source.generics.where_clause, generics, &used_generics);

        let msg_attrs_to_forward = ParsedSylviaAttributes::new(source.attrs.iter())
            .msg_attrs_forward
            .into_iter()
            .filter(|attr| attr.msg_type == ty)
            .collect();

        Some(Self {
            contract_type,
            fields,
            function_name,
            generics: used_generics,
            unused_generics,
            wheres,
            full_where: source.generics.where_clause.as_ref(),
            result: &method.sig.output,
            msg_attr,
            custom,
            msg_attrs_to_forward,
        })
    }

    fn parse_struct_message(source: &ItemImpl, ty: MsgType) -> Option<(&ImplItemFn, MsgAttr)> {
        let mut methods = source.items.iter().filter_map(|item| match item {
            ImplItem::Fn(method) => {
                let attr = ParsedSylviaAttributes::new(method.attrs.iter()).msg_attr?;
                if attr == ty {
                    Some((method, attr))
                } else {
                    None
                }
            }
            _ => None,
        });

        let (method, msg_attr) = if let Some(method) = methods.next() {
            method
        } else {
            if ty == MsgType::Instantiate {
                emit_error!(
                    source.span(), "Missing instantiation message.";
                    note = source.span() => "`sylvia::contract` requires exactly one method marked with `#[sv::msg(instantiation)]` attribute."
                );
            }
            return None;
        };

        if let Some((obsolete, _)) = methods.next() {
            emit_error!(
                obsolete.span(), "More than one instantiation or migration message";
                note = method.span() => "Instantiation/Migration message previously defined here"
            );
        }
        Some((method, msg_attr))
    }

    pub fn emit(&self) -> TokenStream {
        use MsgAttr::*;

        let instantiate_msg = Ident::new("InstantiateMsg", self.function_name.span());
        let migrate_msg = Ident::new("MigrateMsg", self.function_name.span());

        match &self.msg_attr {
            Instantiate { .. } => self.emit_struct(&instantiate_msg),
            Migrate { .. } => self.emit_struct(&migrate_msg),
            _ => {
                emit_error!(Span::mixed_site(), "Invalid message type");
                quote! {}
            }
        }
    }

    pub fn emit_struct(&self, name: &Ident) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            contract_type,
            fields,
            function_name,
            generics,
            unused_generics,
            wheres,
            full_where,
            result,
            msg_attr,
            custom,
            msg_attrs_to_forward,
        } = self;

        let ctx_type = msg_attr
            .msg_type()
            .emit_ctx_type(&custom.query_or_default());
        let fields_names: Vec<_> = fields.iter().map(MsgField::name).collect();
        let parameters = fields.iter().map(MsgField::emit_method_field);
        let fields = fields.iter().map(MsgField::emit_pub);

        let where_clause = as_where_clause(wheres);
        let generics = emit_bracketed_generics(generics);
        let unused_generics = emit_bracketed_generics(unused_generics);
        let msg_attrs_to_forward = msg_attrs_to_forward.iter().map(|attr| &attr.attrs);

        quote! {
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
            #( #[ #msg_attrs_to_forward ] )*
            #[serde(rename_all="snake_case")]
            pub struct #name #generics {
                #(#fields,)*
            }

            impl #generics #name #generics #where_clause {
                pub fn new(#(#parameters,)*) -> Self {
                    Self { #(#fields_names,)* }
                }

                pub fn dispatch #unused_generics(self, contract: &#contract_type, ctx: #ctx_type)
                    #result #full_where
                {
                    let Self { #(#fields_names,)* } = self;
                    contract.#function_name(Into::into(ctx), #(#fields_names,)*).map_err(Into::into)
                }
            }
        }
    }
}
