use syn::fold::Fold;
use syn::{parse_quote, ImplItemType, Path};

use crate::check_generics::GetPath;

pub struct StripSelfPath;

impl Fold for StripSelfPath {
    fn fold_path(&mut self, path: Path) -> Path {
        let segments = path
            .segments
            .into_iter()
            .filter(|segment| segment.ident != "Self")
            .collect();
        syn::fold::fold_path(self, Path { segments, ..path })
    }
}

pub struct AddSelfPath<'a, GenericT>(&'a [&'a GenericT]);

impl<'a, GenericT> AddSelfPath<'a, GenericT> {
    pub fn new(generics: &'a [&'a GenericT]) -> AddSelfPath<GenericT> {
        AddSelfPath(generics)
    }
}

impl<'a, GenericT> Fold for AddSelfPath<'a, GenericT>
where
    GenericT: GetPath,
{
    fn fold_path(&mut self, mut path: Path) -> Path {
        if let Some(pos) = path.segments.iter().position(|segment| {
            self.0.iter().any(|generic| match generic.get_path() {
                Some(path) => Some(&segment.ident) == path.get_ident(),
                None => false,
            })
        }) {
            path.segments.insert(pos, parse_quote! { Self });
        }

        syn::fold::fold_path(self, path)
    }
}

pub struct ReplaceAssociatedType<'a>(&'a [&'a ImplItemType]);

impl<'a> ReplaceAssociatedType<'a> {
    pub fn new(associated_types: &'a [&'a ImplItemType]) -> ReplaceAssociatedType<'a> {
        ReplaceAssociatedType(associated_types)
    }
}

impl<'a> Fold for ReplaceAssociatedType<'a> {
    fn fold_path(&mut self, path: Path) -> Path {
        let segments = path
            .segments
            .into_iter()
            .map(
                |segment| match self.0.iter().find(|generic| segment.ident == generic.ident) {
                    Some(generic) => {
                        let ty = &generic.ty;
                        parse_quote! { #ty }
                    }
                    None => segment,
                },
            )
            .collect();
        syn::fold::fold_path(self, Path { segments, ..path })
    }
}
