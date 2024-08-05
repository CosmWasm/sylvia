use crate::parser::check_generics::{CheckGenerics, GetPath};
use crate::strip_self_path::StripSelfPath;
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Attribute, Ident, Pat, PatType, Type};

/// Representation of single message variant field
#[derive(Debug)]
pub struct MsgField<'a> {
    name: &'a Ident,
    ty: &'a Type,
    stripped_ty: Type,
    attrs: &'a Vec<Attribute>,
}

impl<'a> MsgField<'a> {
    /// Creates new field from trait method argument
    pub fn new<Generic>(
        item: &'a PatType,
        generics_checker: &mut CheckGenerics<Generic>,
    ) -> Option<MsgField<'a>>
    where
        Generic: GetPath + PartialEq,
    {
        let name = match &*item.pat {
            Pat::Ident(p) => Some(&p.ident),
            pat => {
                // TODO: Support pattern arguments, when decorated with argument with item
                // name
                //
                // Eg.
                //
                // ```
                // fn exec_foo(&self, ctx: Ctx, #[sv::msg(name=metadata)] SomeData { addr, sender }: SomeData);
                // ```
                //
                // should expand to enum variant:
                //
                // ```
                // ExecFoo {
                //   metadata: SomeDaa
                // }
                // ```
                emit_error!(pat.span(), "Expected argument name, pattern occurred");
                None
            }
        }?;

        let ty = &item.ty;
        let stripped_ty = StripSelfPath.fold_type((*item.ty).clone());
        let attrs = &item.attrs;
        generics_checker.visit_type(&stripped_ty);

        Some(Self {
            name,
            ty,
            stripped_ty,
            attrs,
        })
    }

    /// Emits message field
    pub fn emit(&self) -> TokenStream {
        let Self {
            name,
            stripped_ty,
            attrs,
            ..
        } = self;

        quote! {
            #(#attrs)*
            #name: #stripped_ty
        }
    }

    /// Emits struct field
    pub fn emit_pub(&self) -> TokenStream {
        let Self {
            name,
            stripped_ty,
            attrs,
            ..
        } = self;

        quote! {
            #(#attrs)*
            pub #name: #stripped_ty
        }
    }

    /// Emits method field
    pub fn emit_method_field(&self) -> TokenStream {
        let Self {
            name, stripped_ty, ..
        } = self;

        quote! {
            #name: #stripped_ty
        }
    }

    pub fn emit_method_field_folded(&self) -> TokenStream {
        let Self { name, ty, .. } = self;

        quote! {
            #name: #ty
        }
    }

    pub fn name(&self) -> &'a Ident {
        self.name
    }
}
