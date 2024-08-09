//! Module defining parsing utilities for Sylvia attributes.
//! All parsing done for the [crate::types] should be done here.

pub mod attributes;
pub mod check_generics;
pub mod entry_point;
pub mod variant_descs;

pub use attributes::{
    ContractErrorAttr, ContractMessageAttr, Custom, Customs, FilteredOverrideEntryPoints, MsgAttr,
    MsgType, OverrideEntryPoint, ParsedSylviaAttributes, SylviaAttribute,
};
use check_generics::{CheckGenerics, GetPath};
pub use entry_point::EntryPointArgs;

use proc_macro_error::emit_error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{FnArg, GenericArgument, ImplItem, ItemImpl, Path, PathArguments, Signature, Token};

use crate::types::msg_field::MsgField;

fn extract_generics_from_path(module: &Path) -> Punctuated<GenericArgument, Token![,]> {
    let generics = module
        .segments
        .last()
        .map(|segment| match segment.arguments.clone() {
            PathArguments::AngleBracketed(generics) => generics.args,
            PathArguments::None => Default::default(),
            PathArguments::Parenthesized(_) => Default::default(),
        })
        .unwrap_or_default();

    generics
}

/// Use to make sure that [contract](crate::contract) is used over implementation containing
/// mandatory `new` method.
pub fn assert_new_method_defined(item: &ItemImpl) {
    const ERROR_NOTE: &str = "`sylvia::contract` requires parameterless `new` method to be defined for dispatch to work correctly.";

    let new = item.items.iter().find_map(|item| match item {
        ImplItem::Fn(method) if method.sig.ident == "new" => Some(method),
        _ => None,
    });

    match new {
        Some(new) if !new.sig.inputs.is_empty() => emit_error!(
            new.sig.inputs, "Parameters not allowed in `new` method.";
            note = ERROR_NOTE;
        ),
        None => {
            emit_error!(
                item, "Missing `new` method in `impl` block.";
                note = ERROR_NOTE;
            )
        }
        _ => (),
    }
}

/// Parses method signature and returns a vector of [`MsgField`].
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
