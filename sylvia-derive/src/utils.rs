use convert_case::Casing;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    parse_quote, GenericArgument, Ident, Path, PathArguments, ReturnType, Type, WhereClause,
    WherePredicate,
};

use crate::parser::check_generics::{CheckGenerics, GetPath};

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

pub trait SvCasing {
    fn to_case(&self, case: convert_case::Case) -> Self;
}

impl SvCasing for Ident {
    fn to_case(&self, case: convert_case::Case) -> Ident {
        let new_name = &self.to_string().to_case(case);
        Ident::new(new_name, self.span())
    }
}
