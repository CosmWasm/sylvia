pub mod attributes;
pub mod contract;
pub mod entry_point;

pub use attributes::{
    ContractErrorAttr, ContractMessageAttr, Custom, Customs, FilteredOverrideEntryPoints, MsgAttr,
    MsgType, OverrideEntryPoint, ParsedSylviaAttributes, SylviaAttribute,
};
pub use entry_point::EntryPointArgs;

use proc_macro_error::emit_error;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse_quote, GenericArgument, Ident, ItemTrait, Path, PathArguments, Token, TraitItem, Type,
};

fn extract_generics_from_path(module: &Path) -> Punctuated<GenericArgument, Token![,]> {
    let generics = module.segments.last().map(|segment| {
        match segment.arguments.clone(){
            PathArguments::AngleBracketed(generics) => {
                generics.args
            },
            PathArguments::None => Default::default(),
            PathArguments::Parenthesized(generics) => {
                emit_error!(
                    generics.span(), "Found paranthesis wrapping generics in `sv::messages` attribute.";
                    note = "Expected `sv::messages` attribute to be in form `#[sv::messages(Path<generics> as Type)]`"
                );
               Default::default()
            }
        }
    }).unwrap_or_default();

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
