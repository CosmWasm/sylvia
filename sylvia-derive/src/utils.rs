use proc_macro2::Span;
use proc_macro_error::emit_error;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Attribute, FnArg, GenericArgument, GenericParam, ImplItem, ItemImpl, ItemTrait, PathArguments,
    PathSegment, ReturnType, Signature, TraitItem, Type, WhereClause, WherePredicate,
};

use crate::check_generics::CheckGenerics;
use crate::message::MsgField;

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

pub fn extract_return_type(ret_type: &ReturnType) -> &PathSegment {
    let ReturnType::Type(_, ty) = ret_type  else {
            unreachable!()
        };

    let Type::Path(type_path) = ty.as_ref()  else {
            unreachable!()
        };
    let segments = &type_path.path.segments;
    assert!(!segments.is_empty());
    let segment = &segments[0];

    // In case of aliased result user need to define the return type by hand
    if segment.ident != "Result" && segment.ident != "StdResult" {
        emit_error!(
            segment.span(),
            "Neither Result nor StdResult found in return type. \
                    You might be using aliased return type. \
                    Please use #[msg(return_type=<your_return_type>)]"
        );
    }
    let PathArguments::AngleBracketed(args) = &segments[0].arguments  else{
            unreachable!()
        };
    let args = &args.args;
    assert!(!args.is_empty());
    let GenericArgument::Type(Type::Path(type_path)) = &args[0] else{
            unreachable!()
        };
    let segments = &type_path.path.segments;
    assert!(!segments.is_empty());

    &segments[0]
}
