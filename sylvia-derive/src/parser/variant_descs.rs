use crate::parser::attributes::VariantAttrForwarding;
use crate::parser::{MsgAttr, ParsedSylviaAttributes};
use syn::{Attribute, ImplItem, ItemImpl, ItemTrait, Signature, TraitItem};

/// Type wrapping common data between [ItemImpl] and [ItemTrait].
pub struct VariantDesc<'a> {
    msg_attr: Option<MsgAttr>,
    attrs_to_forward: Vec<VariantAttrForwarding>,
    sig: &'a Signature,
}

impl<'a> VariantDesc<'a> {
    pub fn new(attrs: &'a [Attribute], sig: &'a Signature) -> Self {
        let sylvia_params = ParsedSylviaAttributes::new(attrs.iter());
        let attrs_to_forward = sylvia_params.variant_attrs_forward;
        let msg_attr = sylvia_params.msg_attr;
        Self {
            msg_attr,
            attrs_to_forward,
            sig,
        }
    }

    pub fn into_sig(self) -> &'a Signature {
        self.sig
    }

    pub fn attr_msg(&self) -> Option<MsgAttr> {
        self.msg_attr.clone()
    }

    pub fn attrs_to_forward(&self) -> Vec<VariantAttrForwarding> {
        self.attrs_to_forward.clone()
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
            ImplItem::Fn(method) => Some(VariantDesc::new(&method.attrs, &method.sig)),
            _ => None,
        }))
    }
}

impl AsVariantDescs for ItemTrait {
    type Iter<'a> = VariantDescs<'a>;

    fn as_variants(&self) -> Self::Iter<'_> {
        Box::new(self.items.iter().filter_map(|item| match item {
            TraitItem::Fn(method) => Some(VariantDesc::new(&method.attrs, &method.sig)),
            _ => None,
        }))
    }
}
