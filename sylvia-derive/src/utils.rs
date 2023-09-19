use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    parse_quote, FnArg, GenericArgument, GenericParam, Path, PathArguments, ReturnType, Signature,
    Type, WhereClause, WherePredicate,
};

use crate::check_generics::CheckGenerics;
use crate::message::MsgField;

#[cfg(not(tarpaulin_include))]
pub fn filter_wheres<'a>(
    clause: &'a Option<WhereClause>,
    generics: &[&GenericParam],
    used_generics: &[&GenericParam],
) -> Vec<&'a WherePredicate> {
    clause
        .as_ref()
        .map(|clause| {
            clause
                .predicates
                .iter()
                .filter(|pred| {
                    let mut generics_checker = CheckGenerics::new(generics);
                    generics_checker.visit_where_predicate(pred);
                    generics_checker
                        .used()
                        .into_iter()
                        .all(|gen| used_generics.contains(&gen))
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn process_fields<'s>(
    sig: &'s Signature,
    generics_checker: &mut CheckGenerics,
) -> Vec<MsgField<'s>> {
    sig.inputs
        .iter()
        .skip(2)
        .filter_map(|arg| match arg {
            FnArg::Receiver(item) => {
                emit_error!(item.span(), "Unexpected `self` argument");
                None
            }

            FnArg::Typed(item) => MsgField::new(item, generics_checker),
        })
        .collect()
}

pub fn extract_return_type(ret_type: &ReturnType) -> &Path {
    let ReturnType::Type(_, ty) = ret_type else {
        unreachable!()
    };

    let Type::Path(type_path) = ty.as_ref() else {
        unreachable!()
    };
    let segments = &type_path.path.segments;
    assert!(!segments.is_empty());
    let segment = &segments[0];

    // In case of aliased result user need to define the return type by hand
    if segment.ident != "Result" && segment.ident != "StdResult" {
        emit_error!(
            segment.span(),
            "Neither Result nor StdResult found in return type. \
                    You might be using aliased return type. \
                    Please use #[msg(return_type=<your_return_type>)]"
        );
    }
    let PathArguments::AngleBracketed(args) = &segments[0].arguments else {
        unreachable!()
    };
    let args = &args.args;
    assert!(!args.is_empty());
    let GenericArgument::Type(Type::Path(type_path)) = &args[0] else {
        unreachable!()
    };

    &type_path.path
}

pub fn as_where_clause(where_predicates: &[&WherePredicate]) -> Option<WhereClause> {
    match where_predicates.is_empty() {
        true => None,
        false => Some(parse_quote! { where #(#where_predicates),* }),
    }
}

pub fn brace_generics(unbonded_generics: &[&GenericParam]) -> TokenStream {
    match unbonded_generics.is_empty() {
        true => quote! {},
        false => quote! { < #(#unbonded_generics,)* > },
    }
}
