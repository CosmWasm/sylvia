use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    parse_quote, ImplItem, ImplItemType, ItemImpl, ItemTrait, TraitItem, TraitItemType, Type,
    WhereClause, WherePredicate,
};

const RESERVED_TYPES: [&str; 3] = ["Error", "QueryC", "ExecC"];

#[derive(Default)]
pub struct AssociatedTypes<'a>(Vec<&'a TraitItemType>);

impl<'a> AssociatedTypes<'a> {
    pub fn new(source: &'a ItemTrait) -> Self {
        let associated_types: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Type(ty) if !RESERVED_TYPES.contains(&ty.ident.to_string().as_str()) => {
                    Some(ty)
                }
                _ => None,
            })
            .collect();

        Self(associated_types)
    }

    pub fn as_where_predicates(&self) -> Vec<WherePredicate> {
        self.0
            .iter()
            .map(|associated| {
                let name = &associated.ident;
                let colon = &associated.colon_token;
                let bound = &associated.bounds;
                parse_quote! { #name #colon #bound }
            })
            .collect()
    }

    pub fn as_where_clause(&self) -> Option<WhereClause> {
        let predicates = self.as_where_predicates();
        if !predicates.is_empty() {
            parse_quote! { where #(#predicates),* }
        } else {
            None
        }
    }

    pub fn as_names(&self) -> Vec<&Ident> {
        self.0.iter().map(|associated| &associated.ident).collect()
    }

    pub fn as_types_declaration(&self) -> &Vec<&TraitItemType> {
        &self.0
    }

    pub fn emit_types_definition(&self) -> Vec<TokenStream> {
        self.as_names()
            .iter()
            .map(|name| quote! { type #name = #name; })
            .collect()
    }

    pub fn emit_contract_predicate(&self, trait_name: &Ident) -> TokenStream {
        let predicate = quote! { ContractT: #trait_name };
        if self.0.is_empty() {
            return predicate;
        }

        let bounds = self.0.iter().map(|associated| {
            let name = &associated.ident;
            quote! { #name = #name }
        });

        quote! {
            #predicate < #(#bounds,)* >
        }
    }
}

#[derive(Default)]
pub struct ImplAssociatedTypes<'a>(Vec<&'a ImplItemType>);

impl<'a> ImplAssociatedTypes<'a> {
    pub fn new(source: &'a ItemImpl) -> Self {
        let associated_types: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Type(ty) if !RESERVED_TYPES.contains(&ty.ident.to_string().as_str()) => {
                    Some(ty)
                }
                _ => None,
            })
            .collect();

        Self(associated_types)
    }

    pub fn as_names(&self) -> Vec<&Ident> {
        self.0.iter().map(|associated| &associated.ident).collect()
    }

    pub fn as_types(&self) -> Vec<&Type> {
        self.0.iter().map(|associated| &associated.ty).collect()
    }

    pub fn as_item_types(&self) -> &Vec<&ImplItemType> {
        &self.0
    }

    pub fn emit_types_declaration(&self) -> Vec<TokenStream> {
        self.as_names()
            .iter()
            .map(|name| quote! { type #name; })
            .collect()
    }
}

pub trait EmitAssociated {
    fn emit_declaration(&self) -> Vec<TokenStream>;
    fn emit_implementation(&self) -> Vec<TokenStream>;
}

impl EmitAssociated for WhereClause {
    fn emit_declaration(&self) -> Vec<TokenStream> {
        self.predicates
            .iter()
            .filter_map(|predicate| match predicate {
                WherePredicate::Type(predicate) => {
                    let bounded_ty = &predicate.bounded_ty;
                    let bounds = &predicate.bounds;
                    let lifetimes = &predicate.lifetimes.as_ref().map(|lf| {
                        let lf = &lf.lifetimes;
                        quote! { < #lf > }
                    });
                    Some(quote! { type #bounded_ty #lifetimes: #bounds; })
                }
                _ => None,
            })
            .collect()
    }

    fn emit_implementation(&self) -> Vec<TokenStream> {
        self.predicates
            .iter()
            .filter_map(|predicate| match predicate {
                WherePredicate::Type(predicate) => {
                    let bounded_ty = &predicate.bounded_ty;
                    let lifetimes = &predicate.lifetimes.as_ref().map(|lf| {
                        let lf = &lf.lifetimes;
                        quote! { < #lf > }
                    });
                    Some(quote! { type #bounded_ty #lifetimes = #bounded_ty; })
                }
                _ => None,
            })
            .collect()
    }
}
