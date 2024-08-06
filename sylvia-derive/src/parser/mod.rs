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
use syn::{
    parse_quote, FnArg, GenericArgument, Ident, ImplItem, ItemImpl, ItemTrait, Path, PathArguments,
    Signature, Token, TraitItem, Type,
};

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

pub fn parse_associated_custom_type(source: &ItemTrait, type_name: &str) -> Option<Type> {
    let trait_name = &source.ident;
    source.items.iter().find_map(|item| match item {
        TraitItem::Type(ty) if ty.ident == type_name => {
            let type_name = Ident::new(type_name, ty.span());
            Some(parse_quote! { <ContractT as #trait_name>:: #type_name})
        }
        _ => None,
    })
}

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
