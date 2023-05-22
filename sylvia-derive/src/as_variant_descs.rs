use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, ImplItem, ItemImpl, ItemTrait, Signature, TraitItem};

pub struct VariantDesc<'a> {
    attrs: &'a Vec<Attribute>,
    sig: &'a Signature,
    span: Span,
}

impl<'a> VariantDesc<'a> {
    pub fn new(attrs: &'a Vec<Attribute>, sig: &'a Signature, span: Span) -> Self {
        Self { attrs, sig, span }
    }

    pub fn into_sig(self) -> &'a Signature {
        self.sig
    }

    pub fn attr_msg(&self) -> Option<&Attribute> {
        self.attrs.iter().find(|attr| attr.path.is_ident("msg"))
    }
}

impl Spanned for VariantDesc<'_> {
    fn span(&self) -> Span {
        self.span
    }
}

pub type VariantDescs<'a> = Box<dyn Iterator<Item = VariantDesc<'a>> + 'a>;

/// Trait for extracting attributes and signature of the methods from `ItemImpl` and `ItemTrait`
/// In most cases these two parameters are being used for preprocessing
/// so to unify the logic we can use this trait
pub trait AsVariantDescs<'a> {
    type Iter: Iterator<Item = VariantDesc<'a>>;
    fn as_variants(&'a self) -> Self::Iter;
}

impl<'a> AsVariantDescs<'a> for ItemImpl {
    type Iter = VariantDescs<'a>;
    fn as_variants(&'a self) -> Self::Iter {
        Box::new(self.items.iter().filter_map(|item| match item {
            ImplItem::Method(method) => {
                Some(VariantDesc::new(&method.attrs, &method.sig, method.span()))
            }
            _ => None,
        }))
    }
}

impl<'a> AsVariantDescs<'a> for ItemTrait {
    type Iter = VariantDescs<'a>;
    fn as_variants(&'a self) -> Self::Iter {
        Box::new(self.items.iter().filter_map(|item| match item {
            TraitItem::Method(method) => {
                Some(VariantDesc::new(&method.attrs, &method.sig, method.span()))
            }
            _ => None,
        }))
    }
}
