use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    parse_quote, FnArg, GenericArgument, GenericParam, Path, PathArguments, ReturnType, Signature,
    Type, WhereClause, WherePredicate,
};

use crate::check_generics::{CheckGenerics, GetPath};
use crate::message::MsgField;

pub fn filter_wheres<'a, Generic: GetPath + PartialEq>(
    clause: &'a Option<WhereClause>,
    generics: &[&Generic],
    used_generics: &[&Generic],
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

/// Filters generic arguments, which are a concrete types,
/// from the generic parameters
///
/// f.e.
/// impl<A, B, C> MyTrait<A, B, D> for MyContract<C>
///
/// where generic parameters are `[A, B, C]` and generic arguments are `[A, B, D]`
/// should return us the `[A, B]`.
pub fn filter_generics<'a>(
    generic_params: &'a [&'a GenericParam],
    generic_args: &'a [&'a GenericArgument],
) -> Vec<&'a GenericParam> {
    generic_params
        .iter()
        .filter(|param| {
            generic_args
                .iter()
                .any(|arg| param.get_path() == arg.get_path())
        })
        .copied()
        .collect()
}

pub fn process_fields<'s, Generic>(
    sig: &'s Signature,
    generics_checker: &mut CheckGenerics<Generic>,
) -> Vec<MsgField<'s>>
where
    Generic: GetPath + PartialEq,
{
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
                    Please use #[sv::msg(return_type=<your_return_type>)]"
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

pub fn emit_bracketed_generics<GenericT: ToTokens>(unbonded_generics: &[GenericT]) -> TokenStream {
    match unbonded_generics.is_empty() {
        true => quote! {},
        false => quote! { < #(#unbonded_generics,)* > },
    }
}
