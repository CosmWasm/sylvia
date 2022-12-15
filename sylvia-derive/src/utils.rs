use proc_macro_error::emit_error;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{FnArg, GenericParam, Signature, WhereClause, WherePredicate};

use crate::check_generics::CheckGenerics;
use crate::message::MsgField;

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
