use syn::fold::{self, Fold};
use syn::punctuated::Punctuated;
use syn::{
    FnArg, ImplItemFn, ItemImpl, ItemTrait, PatType, Receiver, Signature, Token, TraitItemFn,
};

use crate::parser::SylviaAttribute;

/// Utility for stripping all attributes from input before it is emitted
pub struct StripInput;

fn remove_input_attr(inputs: Punctuated<FnArg, Token![,]>) -> Punctuated<FnArg, Token![,]> {
    inputs
        .into_iter()
        .map(|input| match input {
            syn::FnArg::Receiver(rec) if !rec.attrs.is_empty() => {
                let rec = Receiver {
                    attrs: vec![],
                    ..rec
                };
                syn::FnArg::Receiver(rec)
            }
            syn::FnArg::Typed(ty) if !ty.attrs.is_empty() => {
                let ty = PatType {
                    attrs: vec![],
                    ..ty
                };
                syn::FnArg::Typed(ty)
            }
            _ => input,
        })
        .collect()
}

impl Fold for StripInput {
    fn fold_trait_item_fn(&mut self, i: TraitItemFn) -> TraitItemFn {
        let attrs = i
            .attrs
            .into_iter()
            .filter(|attr| SylviaAttribute::new(attr).is_none())
            .collect();

        let inputs = remove_input_attr(i.sig.inputs);
        let sig = Signature { inputs, ..i.sig };
        fold::fold_trait_item_fn(self, TraitItemFn { attrs, sig, ..i })
    }

    fn fold_impl_item_fn(&mut self, i: ImplItemFn) -> ImplItemFn {
        let attrs = i
            .attrs
            .into_iter()
            .filter(|attr| SylviaAttribute::new(attr).is_none())
            .collect();

        let inputs = remove_input_attr(i.sig.inputs);
        let sig = Signature { inputs, ..i.sig };
        fold::fold_impl_item_fn(self, ImplItemFn { attrs, sig, ..i })
    }

    fn fold_item_trait(&mut self, i: ItemTrait) -> ItemTrait {
        let attrs = i
            .attrs
            .into_iter()
            .filter(|attr| SylviaAttribute::new(attr).is_none())
            .collect();

        fold::fold_item_trait(self, ItemTrait { attrs, ..i })
    }

    fn fold_item_impl(&mut self, i: ItemImpl) -> ItemImpl {
        let attrs = i
            .attrs
            .into_iter()
            .filter(|attr| SylviaAttribute::new(attr).is_none())
            .collect();

        fold::fold_item_impl(self, ItemImpl { attrs, ..i })
    }
}
