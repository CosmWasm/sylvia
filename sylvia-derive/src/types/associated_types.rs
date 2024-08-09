use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, ItemTrait, TraitItem, TraitItemType, Type, WhereClause, WherePredicate};

pub const ERROR_TYPE: &str = "Error";
pub const EXEC_TYPE: &str = "ExecC";
pub const QUERY_TYPE: &str = "QueryC";

/// Wrapper around associated types in parsed from a trait.
#[derive(Default)]
pub struct AssociatedTypes<'a>(Vec<&'a TraitItemType>);

impl<'a> AssociatedTypes<'a> {
    pub fn new(source: &'a ItemTrait) -> Self {
        let associated_types: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Type(ty) => Some(ty),
                _ => None,
            })
            .collect();

        Self(associated_types)
    }

    /// Returns [Iterator] over underlying [TraitItemType]s mapped to [Ident]s.
    pub fn as_names(&self) -> impl Iterator<Item = &Ident> {
        self.0.iter().map(|associated| &associated.ident)
    }

    /// Returns [Iterator] over underlying [TraitItemType]s without the `Error` type.
    /// Used for generating generics for generated types.
    pub fn without_error(&self) -> impl Iterator<Item = &TraitItemType> {
        self.0
            .iter()
            .filter(|associated| associated.ident != ERROR_TYPE)
            .cloned()
    }

    /// Returns [WherePredicate] from underlying [TraitItemType]s.
    pub fn as_where_predicates(&self) -> Vec<WherePredicate> {
        self.without_error()
            .map(|associated| {
                let name = &associated.ident;
                let colon = &associated.colon_token;
                let bound = &associated.bounds;
                parse_quote! { #name #colon #bound }
            })
            .collect()
    }

    /// Returns [WhereClause] from underlying [TraitItemType]s.
    pub fn as_where_clause(&self) -> Option<WhereClause> {
        let predicates = self.as_where_predicates();
        if !predicates.is_empty() {
            parse_quote! { where #(#predicates),* }
        } else {
            None
        }
    }

    /// Returns [WhereClause] from underlying [TraitItemType]s.
    pub fn emit_contract_predicate(&self, trait_name: &Ident) -> TokenStream {
        let predicate = quote! { ContractT: #trait_name };
        if self.0.is_empty() {
            return predicate;
        }

        let bounds = self.without_error().map(|associated| {
            let name = &associated.ident;
            quote! { #name = #name }
        });

        quote! {
            #predicate < #(#bounds,)* >
        }
    }

    /// Returns `Some(Type)` if `type_name` is present in the underlying associated types.
    /// Returns `None` if it is not.
    pub fn emit_contract_custom_type_accessor(
        &self,
        trait_name: &Ident,
        type_name: &str,
    ) -> Option<Type> {
        match self
            .as_names()
            .find(|name| name.to_string().as_str() == type_name)
        {
            Some(name) => {
                let type_name = Ident::new(type_name, name.span());
                Some(parse_quote! { <ContractT as #trait_name>:: #type_name})
            }
            None => None,
        }
    }
}

/// Trait defining convertion to [Ident] and [WherePredicate].
pub trait ItemType {
    fn as_name(&self) -> &Ident;
    fn as_where_predicate(&self) -> WherePredicate;
}

impl ItemType for TraitItemType {
    fn as_name(&self) -> &Ident {
        &self.ident
    }

    fn as_where_predicate(&self) -> WherePredicate {
        let name = &self.ident;
        let colon = &self.colon_token;
        let bound = &self.bounds;
        parse_quote! { #name #colon #bound }
    }
}

/// Trait generating associated types.
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
