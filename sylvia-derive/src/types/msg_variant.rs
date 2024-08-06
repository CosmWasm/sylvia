use crate::crate_module;
use crate::fold::StripSelfPath;
use crate::parser::attributes::VariantAttrForwarding;
use crate::parser::check_generics::{CheckGenerics, GetPath};
use crate::parser::variant_descs::VariantDescs;
use crate::parser::{process_fields, MsgAttr, MsgType};
use crate::utils::{extract_return_type, filter_wheres, SvCasing};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::fold::Fold;
use syn::visit::Visit;
use syn::{parse_quote, Ident, Signature, Type, WhereClause, WherePredicate};

use super::msg_field::MsgField;

/// Representation of whole message variant
#[derive(Debug)]
pub struct MsgVariant<'a> {
    name: Ident,
    function_name: &'a Ident,
    fields: Vec<MsgField<'a>>,
    /// Type extracted only in case of `Query` and used in `cosmwasm_schema::QueryResponses`
    /// `returns` attribute.
    return_type: Option<Type>,
    msg_type: MsgType,
    attrs_to_forward: Vec<VariantAttrForwarding>,
}

impl<'a> MsgVariant<'a> {
    /// Creates new message variant from trait method
    pub fn new<Generic>(
        sig: &'a Signature,
        generics_checker: &mut CheckGenerics<Generic>,
        msg_attr: MsgAttr,
        attrs_to_forward: Vec<VariantAttrForwarding>,
    ) -> MsgVariant<'a>
    where
        Generic: GetPath + PartialEq,
    {
        let function_name = &sig.ident;

        let name = function_name.to_case(Case::UpperCamel);
        let fields = process_fields(sig, generics_checker);
        let msg_type = msg_attr.msg_type();

        let return_type = if let MsgAttr::Query { resp_type, .. } = msg_attr {
            match resp_type {
                Some(resp_type) => {
                    let resp_type = parse_quote! { #resp_type };
                    generics_checker.visit_type(&resp_type);
                    Some(resp_type)
                }
                None => {
                    let return_type = extract_return_type(&sig.output);
                    let stripped_return_type = StripSelfPath.fold_path(return_type.clone());
                    generics_checker.visit_path(&stripped_return_type);
                    Some(parse_quote! { #return_type })
                }
            }
        } else {
            None
        };

        Self {
            name,
            function_name,
            fields,
            return_type,
            msg_type,
            attrs_to_forward,
        }
    }

    /// Emits message variant
    pub fn emit(&self) -> TokenStream {
        let Self {
            name,
            fields,
            msg_type,
            return_type,
            attrs_to_forward,
            ..
        } = self;
        let fields = fields.iter().map(MsgField::emit);
        let returns_attribute = msg_type.emit_returns_attribute(return_type);
        let attrs_to_forward = attrs_to_forward.iter().map(|attr| &attr.attrs);

        quote! {
            #returns_attribute
            #( #[ #attrs_to_forward ] )*
            #name {
                #(#fields,)*
            }
        }
    }

    /// Emits match leg dispatching against this variant. Assumes enum variants are imported into the
    /// scope. Dispatching is performed by calling the function this variant is build from on the
    /// `contract` variable, with `ctx` as its first argument - both of them should be in scope.
    pub fn emit_dispatch_leg(&self) -> TokenStream {
        let Self {
            name,
            fields,
            function_name,
            msg_type,
            ..
        } = self;

        let args: Vec<_> = fields
            .iter()
            .zip(1..)
            .map(|(field, num)| Ident::new(&format!("field{}", num), field.name().span()))
            .collect();

        let fields = fields
            .iter()
            .map(MsgField::name)
            .zip(args.clone())
            .map(|(field, num_field)| quote!(#field : #num_field));

        let method_call = msg_type.emit_dispatch_leg(function_name, &args);

        quote! {
            #name {
                #(#fields,)*
            } => #method_call
        }
    }

    /// Emits variants constructors. Constructors names are variants names in snake_case.
    pub fn emit_variants_constructors(&self) -> TokenStream {
        let Self { name, fields, .. } = self;

        let method_name = name.to_case(Case::Snake);
        let parameters = fields.iter().map(MsgField::emit_method_field);
        let arguments = fields.iter().map(MsgField::name);

        quote! {
            pub fn #method_name( #(#parameters),*) -> Self {
                Self :: #name { #(#arguments),* }
            }
        }
    }

    pub fn as_fields_names(&self) -> Vec<&Ident> {
        self.fields.iter().map(MsgField::name).collect()
    }

    pub fn emit_method_field(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(MsgField::emit_method_field)
            .collect()
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn function_name(&self) -> &Ident {
        self.function_name
    }

    pub fn fields(&self) -> &Vec<MsgField> {
        &self.fields
    }

    pub fn msg_type(&self) -> &MsgType {
        &self.msg_type
    }

    pub fn return_type(&self) -> &Option<Type> {
        &self.return_type
    }
}

#[derive(Debug)]
pub struct MsgVariants<'a, Generic> {
    variants: Vec<MsgVariant<'a>>,
    used_generics: Vec<&'a Generic>,
    unused_generics: Vec<&'a Generic>,
    where_predicates: Vec<&'a WherePredicate>,
    msg_ty: MsgType,
}

impl<'a, Generic> MsgVariants<'a, Generic>
where
    Generic: GetPath + PartialEq + ToTokens,
{
    pub fn new(
        source: VariantDescs<'a>,
        msg_ty: MsgType,
        all_generics: &'a [&'a Generic],
        unfiltered_where_clause: &'a Option<WhereClause>,
    ) -> Self {
        let mut generics_checker = CheckGenerics::new(all_generics);
        let variants: Vec<_> = source
            .filter_map(|variant_desc| {
                let msg_attr: MsgAttr = variant_desc.attr_msg()?;
                let attrs_to_forward = variant_desc.attrs_to_forward();

                if msg_attr.msg_type() != msg_ty {
                    return None;
                }

                Some(MsgVariant::new(
                    variant_desc.into_sig(),
                    &mut generics_checker,
                    msg_attr,
                    attrs_to_forward,
                ))
            })
            .collect();

        let (used_generics, unused_generics) = generics_checker.used_unused();
        let where_predicates = filter_wheres(unfiltered_where_clause, all_generics, &used_generics);

        Self {
            variants,
            used_generics,
            unused_generics,
            where_predicates,
            msg_ty,
        }
    }

    pub fn where_clause(&self) -> Option<WhereClause> {
        let where_predicates = &self.where_predicates;
        if !where_predicates.is_empty() {
            Some(parse_quote! { where #(#where_predicates),* })
        } else {
            None
        }
    }

    pub fn variants(&self) -> impl Iterator<Item = &MsgVariant> {
        self.variants.iter()
    }

    pub fn used_generics(&self) -> &Vec<&'a Generic> {
        &self.used_generics
    }

    pub fn unused_generics(&self) -> &Vec<&'a Generic> {
        &self.unused_generics
    }

    pub fn msg_ty(&self) -> MsgType {
        self.msg_ty
    }

    pub fn emit_phantom_match_arm(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self { used_generics, .. } = self;
        if used_generics.is_empty() {
            return quote! {};
        }
        quote! {
            _Phantom(_) => Err(#sylvia ::cw_std::StdError::generic_err("Phantom message should not be constructed.")).map_err(Into::into),
        }
    }

    pub fn emit_dispatch_legs(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants
            .iter()
            .map(|variant| variant.emit_dispatch_leg())
    }

    pub fn as_names_snake_cased(&self) -> Vec<String> {
        self.variants
            .iter()
            .map(|variant| variant.name.to_string().to_case(Case::Snake))
            .collect()
    }

    pub fn emit_constructors(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants
            .iter()
            .map(MsgVariant::emit_variants_constructors)
    }

    pub fn emit(&self) -> impl Iterator<Item = TokenStream> + '_ {
        self.variants.iter().map(MsgVariant::emit)
    }

    pub fn get_only_variant(&self) -> Option<&MsgVariant> {
        self.variants.first()
    }

    pub fn emit_phantom_variant(&self) -> TokenStream {
        let Self {
            msg_ty,
            used_generics,
            ..
        } = self;

        if used_generics.is_empty() {
            return quote! {};
        }

        let return_attr = match msg_ty {
            MsgType::Query => quote! { #[returns((#(#used_generics,)*))] },
            _ => quote! {},
        };

        quote! {
            #[serde(skip)]
            #return_attr
            _Phantom(std::marker::PhantomData<( #(#used_generics,)* )>),
        }
    }
}
