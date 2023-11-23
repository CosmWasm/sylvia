use syn::visit::Visit;
use syn::{parse_quote, GenericArgument, GenericParam, Type};

pub trait GetPath {
    fn get_path(&self) -> Option<syn::Path>;
}

impl GetPath for GenericParam {
    fn get_path(&self) -> Option<syn::Path> {
        match self {
            GenericParam::Type(ty) => {
                let ident = &ty.ident;
                Some(parse_quote! { #ident })
            }
            _ => None,
        }
    }
}

impl GetPath for GenericArgument {
    fn get_path(&self) -> Option<syn::Path> {
        match self {
            GenericArgument::Type(Type::Path(path)) => {
                let path = &path.path;
                Some(parse_quote! { #path })
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CheckGenerics<'g, Generic> {
    generics: &'g [&'g Generic],
    used: Vec<&'g Generic>,
}

impl<'g, Generic> CheckGenerics<'g, Generic>
where
    Generic: GetPath + PartialEq,
{
    pub fn new(generics: &'g [&'g Generic]) -> Self {
        Self {
            generics,
            used: vec![],
        }
    }

    pub fn used(self) -> Vec<&'g Generic> {
        self.used
    }

    /// Returns split between used and unused generics
    pub fn used_unused(self) -> (Vec<&'g Generic>, Vec<&'g Generic>) {
        let unused = self
            .generics
            .iter()
            .filter(|gen| !self.used.contains(*gen))
            .copied()
            .collect();

        (self.used, unused)
    }
}

impl<'ast, 'g, Generic> Visit<'ast> for CheckGenerics<'g, Generic>
where
    Generic: GetPath + PartialEq,
{
    fn visit_path(&mut self, p: &'ast syn::Path) {
        if let Some(gen) = self
            .generics
            .iter()
            .find(|gen| gen.get_path().as_ref() == Some(p))
        {
            self.used.push(gen);
        }

        // Default visit implementation - visiting path deeper
        for el in &p.segments {
            self.visit_path_segment(el);
        }
    }
}
