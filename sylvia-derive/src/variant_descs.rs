use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, ImplItem, ItemImpl, ItemTrait, Signature, TraitItem};

pub struct VariantDesc<'a> {
    attrs: &'a [Attribute],
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
        self.attrs.iter().find(|attr| attr.path().is_ident("msg"))
    }

    pub fn span(&self) -> Span {
        self.span
    }
}

pub type VariantDescs<'a> = Box<dyn Iterator<Item = VariantDesc<'a>> + 'a>;

/// Trait for extracting attributes and signature of the methods from `ItemImpl` and `ItemTrait`
/// In most cases these two parameters are being used for preprocessing
/// so to unify the logic we can use this trait
pub trait AsVariantDescs {
    type Iter<'a>: Iterator<Item = VariantDesc<'a>>
    where
        Self: 'a;

    fn as_variants(&self) -> Self::Iter<'_>;
}

impl AsVariantDescs for ItemImpl {
    type Iter<'a> = VariantDescs<'a>;

    fn as_variants(&self) -> Self::Iter<'_> {
        Box::new(self.items.iter().filter_map(|item| match item {
            ImplItem::Fn(method) => {
                Some(VariantDesc::new(&method.attrs, &method.sig, method.span()))
            }
            _ => None,
        }))
    }
}

impl AsVariantDescs for ItemTrait {
    type Iter<'a> = VariantDescs<'a>;

    fn as_variants(&self) -> Self::Iter<'_> {
        Box::new(self.items.iter().filter_map(|item| match item {
            TraitItem::Fn(method) => {
                Some(VariantDesc::new(&method.attrs, &method.sig, method.span()))
            }
            _ => None,
        }))
    }
}
