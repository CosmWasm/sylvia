use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::fold::Fold;
use syn::{parse_quote, GenericParam, Ident, ItemImpl, Type, WhereClause};

use crate::crate_module;
use crate::fold::StripGenerics;
use crate::parser::attributes::features::SylviaFeatures;
use crate::parser::attributes::msg::MsgType;
use crate::parser::variant_descs::AsVariantDescs;
use crate::parser::{
    EntryPointArgs, FilteredOverrideEntryPoints, OverrideEntryPoint, ParsedSylviaAttributes,
};
use crate::types::msg_variant::MsgVariants;

/// Preprocessed [`entry_points`](crate::entry_points) macro input.
///
/// Generates `entry_points` module containing:
///     - instantiate, execute, query and sudo entry points by default
///     - migrate and reply entry points if respective messages are defined
pub struct EntryPointInput<'a> {
    item: &'a ItemImpl,
    args: EntryPointArgs,
}

impl<'a> EntryPointInput<'a> {
    pub fn new(item: &'a ItemImpl, args: EntryPointArgs, attr_span: Span) -> Self {
        let instantiate =
            MsgVariants::<GenericParam>::new(item.as_variants(), MsgType::Instantiate, &[], &None);

        if args.generics.len() != item.generics.params.len() {
            emit_error!(
                attr_span,
                "Missing concrete types.";
                note = "For every generic type in the contract, a concrete type must be provided in `#[entry_points(generics<T1, T2, ...>)]`.";
            );
        }

        if instantiate.get_only_variant().is_none() {
            emit_error!(
                attr_span, "Missing instantiation message.";
                note = "`sylvia::entry_points` requires exactly one method marked with `#[sv::msg(instantiation)]` attribute.";
                note = "Make sure you implemented the `entry_points` macro above the `contract` macro."
            );
        }

        Self { item, args }
    }

    /// Process the input and generate the entry points code.
    pub fn process(&self) -> TokenStream {
        let Self { item, args } = self;

        EntryPoints::new(item, args).emit()
    }
}

/// Defines logic for generating entry points.
///
/// By default generates entry points for `instantiate`, `execute`, `query` and `sudo` messages.
/// Generates `reply` and `migrate` entry points if respective messages are defined.
pub struct EntryPoints<'a> {
    source: &'a ItemImpl,
    name: Type,
    error: Type,
    reply: Option<Ident>,
    override_entry_points: Vec<OverrideEntryPoint>,
    generics: Vec<&'a GenericParam>,
    where_clause: &'a Option<WhereClause>,
    attrs: &'a EntryPointArgs,
    _sv_features: SylviaFeatures,
}

impl<'a> EntryPoints<'a> {
    pub fn new(source: &'a ItemImpl, attrs: &'a EntryPointArgs) -> Self {
        let name = StripGenerics.fold_type(*source.self_ty.clone());
        let parsed_attrs = ParsedSylviaAttributes::new(source.attrs.iter());
        let override_entry_points = parsed_attrs.override_entry_point_attrs;
        let sv_features = parsed_attrs.sv_features;

        let error = parsed_attrs.error_attrs.unwrap_or_default().error;

        let generics: Vec<_> = source.generics.params.iter().collect();
        let where_clause = &source.generics.where_clause;

        let reply =
            MsgVariants::<GenericParam>::new(source.as_variants(), MsgType::Reply, &[], &None)
                .variants()
                .map(|variant| variant.function_name().clone())
                .next();

        Self {
            source,
            name,
            error,
            reply,
            override_entry_points,
            generics,
            where_clause,
            attrs,
            _sv_features: sv_features,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let Self {
            source,
            reply,
            override_entry_points,
            generics,
            where_clause,
            ..
        } = self;

        let entry_points = [
            MsgType::Instantiate,
            MsgType::Exec,
            MsgType::Query,
            MsgType::Sudo,
        ]
        .into_iter()
        .map(
            |msg_ty| match override_entry_points.get_entry_point(msg_ty) {
                Some(_) => quote! {},
                None => self.emit_default_entry_point(msg_ty),
            },
        );

        let is_migrate = MsgVariants::new(
            source.as_variants(),
            MsgType::Migrate,
            generics,
            where_clause,
        )
        .get_only_variant()
        .is_some();

        let migrate_not_overridden = override_entry_points
            .get_entry_point(MsgType::Migrate)
            .is_none();

        let migrate = if migrate_not_overridden && is_migrate {
            self.emit_default_entry_point(MsgType::Migrate)
        } else {
            quote! {}
        };

        let reply_ep = override_entry_points
            .get_entry_point(MsgType::Reply)
            .map(|_| quote! {})
            .unwrap_or_else(|| {
                if reply.is_some() {
                    self.emit_default_entry_point(MsgType::Reply)
                } else {
                    quote! {}
                }
            });

        quote! {
            pub mod entry_points {
                use super::*;

                #(#entry_points)*

                #migrate

                #reply_ep
            }
        }
    }

    fn emit_default_entry_point(&self, msg_ty: MsgType) -> TokenStream {
        let Self {
            name, error, attrs, ..
        } = self;
        let sylvia = crate_module();

        let attr_generics = &attrs.generics;
        let (contract, contract_turbofish) = if attr_generics.is_empty() {
            (quote! { #name }, quote! { #name })
        } else {
            (
                quote! { #name < #attr_generics > },
                quote! { #name :: < #attr_generics > },
            )
        };

        let custom_msg: Type =
            parse_quote! { < #contract as #sylvia ::types::ContractApi > :: CustomMsg };
        let custom_query: Type =
            parse_quote! { < #contract as #sylvia ::types::ContractApi > :: CustomQuery };

        let result = msg_ty.emit_result_type(&custom_msg, error);
        let params = msg_ty.emit_ctx_params(&custom_query);
        let values = msg_ty.emit_ctx_values();
        let ep_name = msg_ty.emit_ep_name();
        let associated_name = msg_ty.as_accessor_wrapper_name();
        let msg = match msg_ty {
            MsgType::Reply => quote! { msg: #sylvia ::cw_std::Reply },
            _ => quote! { msg: < #contract as #sylvia ::types::ContractApi> :: #associated_name },
        };
        let dispatch = match msg_ty {
            MsgType::Reply => quote! {
                let contract = #contract_turbofish ::new();
                sv::dispatch_reply(deps, env, msg, contract).map_err(Into::into)
            },
            _ => quote! {
                msg.dispatch(& #contract_turbofish ::new() , ( #values )).map_err(Into::into)
            },
        };

        let cw_std = quote! { #sylvia ::cw_std }.to_string();

        quote! {
            #[#sylvia ::cw_std::entry_point(crate = #cw_std )]
            pub fn #ep_name (
                #params ,
                #msg
            ) -> #result {
                #dispatch
            }
        }
    }
}
