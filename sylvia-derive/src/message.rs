use crate::check_generics::CheckGenerics;
use crate::crate_module;
use crate::parser::{
    parse_struct_message, ContractErrorAttr, ContractMessageAttr, InterfaceArgs, MsgAttr, MsgType,
};
use crate::strip_generics::StripGenerics;
use crate::utils::{extract_return_type, filter_wheres, process_fields};
use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use syn::fold::Fold;
use syn::parse::{Parse, Parser};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    parse_quote, Attribute, GenericParam, Ident, ImplItem, ItemImpl, ItemTrait, Pat, PatType,
    ReturnType, Signature, TraitItem, Type, WhereClause, WherePredicate,
};

/// Representation of single struct message
pub struct StructMessage<'a> {
    contract_type: &'a Type,
    fields: Vec<MsgField<'a>>,
    function_name: &'a Ident,
    generics: Vec<&'a GenericParam>,
    unused_generics: Vec<&'a GenericParam>,
    wheres: Vec<&'a WherePredicate>,
    full_where: Option<&'a WhereClause>,
    result: &'a ReturnType,
    msg_attr: MsgAttr,
}

impl<'a> StructMessage<'a> {
    /// Creates new struct message of given type from impl block
    pub fn new(
        source: &'a ItemImpl,
        ty: MsgType,
        generics: &'a [&'a GenericParam],
    ) -> Option<StructMessage<'a>> {
        let mut generics_checker = CheckGenerics::new(generics);

        let contract_type = &source.self_ty;

        let parsed = parse_struct_message(source, ty);
        let Some((method, msg_attr)) = parsed else {
            return None;
        };

        let function_name = &method.sig.ident;
        let fields = process_fields(&method.sig, &mut generics_checker);
        let (used_generics, unused_generics) = generics_checker.used_unused();
        let wheres = filter_wheres(&source.generics.where_clause, generics, &used_generics);

        Some(Self {
            contract_type,
            fields,
            function_name,
            generics: used_generics,
            unused_generics,
            wheres,
            full_where: source.generics.where_clause.as_ref(),
            result: &method.sig.output,
            msg_attr,
        })
    }

    pub fn emit(&self) -> TokenStream {
        use MsgAttr::*;

        match &self.msg_attr {
            Instantiate { name } => self.emit_struct(name),
            Migrate { name } => self.emit_struct(name),
            _ => {
                emit_error!(Span::mixed_site(), "Invalid message type");
                quote! {}
            }
        }
    }

    pub fn emit_struct(&self, name: &Ident) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            contract_type,
            fields,
            function_name,
            generics,
            unused_generics,
            wheres,
            full_where,
            result,
            msg_attr,
        } = self;

        let where_clause = if !wheres.is_empty() {
            quote! {
                where #(#wheres,)*
            }
        } else {
            quote! {}
        };

        let ctx_type = msg_attr.msg_type().emit_ctx_type();
        let fields_names: Vec<_> = fields.iter().map(MsgField::name).collect();
        let parameters = fields.iter().map(|field| {
            let name = field.name;
            let ty = field.ty;
            quote! { #name : #ty}
        });
        let fields = fields.iter().map(MsgField::emit);

        let generics = if generics.is_empty() {
            quote! {}
        } else {
            quote! {
                <#(#generics,)*>
            }
        };

        let unused_generics = if unused_generics.is_empty() {
            quote! {}
        } else {
            quote! {
                <#(#unused_generics,)*>
            }
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #[allow(clippy::derive_partial_eq_without_eq)]
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                #[serde(rename_all="snake_case")]
                pub struct #name #generics #where_clause {
                    #(pub #fields,)*
                }

                impl #generics #name #generics #where_clause {
                    pub fn new(#(#parameters,)*) -> Self {
                        Self { #(#fields_names,)* }
                    }

                    pub fn dispatch #unused_generics(self, contract: &#contract_type, ctx: #ctx_type)
                        #result #full_where
                    {
                        let Self { #(#fields_names,)* } = self;
                        contract.#function_name(Into::into(ctx), #(#fields_names,)*).map_err(Into::into)
                    }
                }
            }
        }
    }
}

/// Representation of single enum message
pub struct EnumMessage<'a> {
    name: &'a Ident,
    trait_name: &'a Ident,
    variants: Vec<MsgVariant<'a>>,
    generics: Vec<&'a GenericParam>,
    unused_generics: Vec<&'a GenericParam>,
    all_generics: &'a [&'a GenericParam],
    wheres: Vec<&'a WherePredicate>,
    full_where: Option<&'a WhereClause>,
    msg_ty: MsgType,
    args: &'a InterfaceArgs,
}

impl<'a> EnumMessage<'a> {
    pub fn new(
        name: &'a Ident,
        source: &'a ItemTrait,
        ty: MsgType,
        generics: &'a [&'a GenericParam],
        args: &'a InterfaceArgs,
    ) -> Self {
        let trait_name = &source.ident;

        let mut generics_checker = CheckGenerics::new(generics);
        let variants: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                TraitItem::Method(method) => {
                    let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                    let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                        Ok(attr) => attr,
                        Err(err) => {
                            emit_error!(method.span(), err);
                            return None;
                        }
                    };

                    if attr == ty {
                        Some(MsgVariant::new(
                            &method.sig,
                            &mut generics_checker,
                            name,
                            attr,
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        let (used_generics, unused_generics) = generics_checker.used_unused();
        let wheres = filter_wheres(&source.generics.where_clause, generics, &used_generics);

        Self {
            name,
            trait_name,
            variants,
            generics: used_generics,
            unused_generics,
            all_generics: generics,
            wheres,
            full_where: source.generics.where_clause.as_ref(),
            msg_ty: ty,
            args,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            name,
            trait_name,
            variants,
            generics,
            unused_generics,
            all_generics,
            wheres,
            full_where,
            msg_ty,
            args,
        } = self;

        let match_arms = variants
            .iter()
            .map(|variant| variant.emit_dispatch_leg(*msg_ty));
        let mut msgs: Vec<String> = variants
            .iter()
            .map(|var| var.name.to_string().to_case(Case::Snake))
            .collect();
        msgs.sort();
        let msgs_cnt = msgs.len();
        let variants_constructors = variants.iter().map(MsgVariant::emit_variants_constructors);
        let variants = variants.iter().map(MsgVariant::emit);
        let where_clause = if !wheres.is_empty() {
            quote! {
                where #(#wheres,)*
            }
        } else {
            quote! {}
        };

        let ctx_type = msg_ty.emit_ctx_type();
        let dispatch_type = msg_ty.emit_result_type(&args.msg_type, &parse_quote!(C::Error));

        let all_generics = if all_generics.is_empty() {
            quote! {}
        } else {
            quote! { <#(#all_generics,)*> }
        };

        let generics = if generics.is_empty() {
            quote! {}
        } else {
            quote! { <#(#generics,)*> }
        };

        let unique_enum_name = Ident::new(&format!("{}{}", trait_name, name), name.span());

        #[cfg(not(tarpaulin_include))]
        let enum_declaration = match name.to_string().as_str() {
            "QueryMsg" => {
                quote! {
                    #[allow(clippy::derive_partial_eq_without_eq)]
                    #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, cosmwasm_schema::QueryResponses)]
                    #[serde(rename_all="snake_case")]
                    pub enum #unique_enum_name #generics #where_clause {
                        #(#variants,)*
                    }
                    pub type #name #generics = #unique_enum_name #generics;
                }
            }
            _ => {
                quote! {
                    #[allow(clippy::derive_partial_eq_without_eq)]
                    #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                    #[serde(rename_all="snake_case")]
                    pub enum #unique_enum_name #generics #where_clause {
                        #(#variants,)*
                    }
                    pub type #name #generics = #unique_enum_name #generics;
                }
            }
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #enum_declaration

                impl #generics #unique_enum_name #generics #where_clause {
                    pub fn dispatch<C: #trait_name #all_generics, #(#unused_generics,)*>(self, contract: &C, ctx: #ctx_type)
                        -> #dispatch_type #full_where
                    {
                        use #unique_enum_name::*;

                        match self {
                            #(#match_arms,)*
                        }
                    }
                    pub const fn messages() -> [&'static str; #msgs_cnt] {
                        [#(#msgs,)*]
                    }
                    #(#variants_constructors)*
                }
            }
        }
    }
}

/// Representation of single enum message
pub struct ContractEnumMessage<'a> {
    name: &'a Ident,
    variants: Vec<MsgVariant<'a>>,
    msg_ty: MsgType,
    contract: &'a Type,
    error: &'a Type,
}

impl<'a> ContractEnumMessage<'a> {
    pub fn new(
        name: &'a Ident,
        source: &'a ItemImpl,
        ty: MsgType,
        generics: &'a [&'a GenericParam],
        error: &'a Type,
    ) -> Self {
        let mut generics_checker = CheckGenerics::new(generics);
        let variants: Vec<_> = source
            .items
            .iter()
            .filter_map(|item| match item {
                ImplItem::Method(method) => {
                    let msg_attr = method.attrs.iter().find(|attr| attr.path.is_ident("msg"))?;
                    let attr = match MsgAttr::parse.parse2(msg_attr.tokens.clone()) {
                        Ok(attr) => attr,
                        Err(err) => {
                            emit_error!(method.span(), err);
                            return None;
                        }
                    };

                    if attr == ty {
                        Some(MsgVariant::new(
                            &method.sig,
                            &mut generics_checker,
                            name,
                            attr,
                        ))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .collect();

        Self {
            name,
            variants,
            msg_ty: ty,
            contract: &source.self_ty,
            error,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            name,
            variants,
            msg_ty,
            contract,
            error,
        } = self;

        let match_arms = variants
            .iter()
            .map(|variant| variant.emit_dispatch_leg(*msg_ty));
        let mut msgs: Vec<String> = variants
            .iter()
            .map(|var| var.name.to_string().to_case(Case::Snake))
            .collect();
        msgs.sort();
        let msgs_cnt = msgs.len();
        let variants_constructors = variants.iter().map(MsgVariant::emit_variants_constructors);
        let variants = variants.iter().map(MsgVariant::emit);

        let ctx_type = msg_ty.emit_ctx_type();
        let contract = StripGenerics.fold_type((*contract).clone());
        let ret_type = msg_ty.emit_result_type(&None, error);

        #[cfg(not(tarpaulin_include))]
        let enum_declaration = match name.to_string().as_str() {
            "QueryMsg" => {
                quote! {
                        #[allow(clippy::derive_partial_eq_without_eq)]
                        #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, cosmwasm_schema::QueryResponses)]
                        #[serde(rename_all="snake_case")]
                        pub enum #name {
                            #(#variants,)*
                        }
                }
            }
            _ => {
                quote! {
                        #[allow(clippy::derive_partial_eq_without_eq)]
                        #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                        #[serde(rename_all="snake_case")]
                        pub enum #name {
                            #(#variants,)*
                    }
                }
            }
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #enum_declaration

                impl #name {
                    pub fn dispatch(self, contract: &#contract, ctx: #ctx_type) -> #ret_type {
                        use #name::*;

                        match self {
                            #(#match_arms,)*
                        }
                    }
                    pub const fn messages() -> [&'static str; #msgs_cnt] {
                        [#(#msgs,)*]
                    }

                    #(#variants_constructors)*
                }
            }
        }
    }
}

/// Representation of whole message variant
pub struct MsgVariant<'a> {
    name: Ident,
    function_name: &'a Ident,
    // With https://github.com/rust-lang/rust/issues/63063 this could be just an iterator over
    // `MsgField<'a>`
    fields: Vec<MsgField<'a>>,
    return_type: TokenStream,
    message_type: &'a Ident,
}

impl<'a> MsgVariant<'a> {
    /// Creates new message variant from trait method
    pub fn new(
        sig: &'a Signature,
        generics_checker: &mut CheckGenerics,
        message_type: &'a Ident,
        msg_attr: MsgAttr,
    ) -> MsgVariant<'a> {
        let function_name = &sig.ident;

        let name = Ident::new(
            &function_name.to_string().to_case(Case::UpperCamel),
            function_name.span(),
        );
        let fields = process_fields(sig, generics_checker);

        let return_type = if let MsgAttr::Query { resp_type } = msg_attr {
            match resp_type {
                Some(resp_type) => quote! {#resp_type},
                None => {
                    let return_type = extract_return_type(&sig.output);
                    quote! {#return_type}
                }
            }
        } else {
            quote! {}
        };

        Self {
            name,
            function_name,
            fields,
            return_type,
            message_type,
        }
    }

    /// Emits message variant
    pub fn emit(&self) -> TokenStream {
        let Self { name, fields, .. } = self;
        let fields = fields.iter().map(MsgField::emit);
        let return_type = &self.return_type;

        if self.message_type == "QueryMsg" {
            #[cfg(not(tarpaulin_include))]
            {
                quote! {
                    #[returns(#return_type)]
                    #name {
                        #(#fields,)*
                    }
                }
            }
        } else {
            #[cfg(not(tarpaulin_include))]
            {
                quote! {
                    #name {
                        #(#fields,)*
                    }
                }
            }
        }
    }

    /// Emits match leg dispatching against this variant. Assumes enum variants are imported into the
    /// scope. Dispatching is performed by calling the function this variant is build from on the
    /// `contract` variable, with `ctx` as its first argument - both of them should be in scope.
    pub fn emit_dispatch_leg(&self, msg_attr: MsgType) -> TokenStream {
        use MsgType::*;

        let Self {
            name,
            fields,
            function_name,
            ..
        } = self;
        let args = fields
            .iter()
            .zip(1..)
            .map(|(field, num)| Ident::new(&format!("field{}", num), field.name.span()));

        let fields = fields
            .iter()
            .map(|field| field.name)
            .zip(args.clone())
            .map(|(field, num_field)| quote!(#field : #num_field));

        match msg_attr {
            Exec => quote! {
                #name {
                    #(#fields,)*
                } => contract.#function_name(Into::into(ctx), #(#args),*).map_err(Into::into)
            },
            Query => quote! {
                #name {
                    #(#fields,)*
                } => cosmwasm_std::to_binary(&contract.#function_name(Into::into(ctx), #(#args),*)?).map_err(Into::into)
            },
            Instantiate | Migrate => {
                emit_error!(name.span(), "Instantiation and Migrate messages not supported on traits, they should be defined on contracts directly");
                quote! {}
            }
        }
    }

    /// Emits variants constructors. Constructors names are variants names in snake_case.
    pub fn emit_variants_constructors(&self) -> TokenStream {
        let Self { name, fields, .. } = self;

        let method_name = name.to_string().to_case(Case::Snake);
        let method_name = Ident::new(&method_name, name.span());

        let parameters = fields.iter().map(|field| {
            let name = field.name;
            let ty = field.ty;
            quote! { #name : #ty}
        });
        let arguments = fields.iter().map(|field| field.name);

        quote! {
            pub fn #method_name( #(#parameters),*) -> Self {
                Self :: #name { #(#arguments),* }
            }
        }
    }
}

/// Representation of single message variant field
pub struct MsgField<'a> {
    name: &'a Ident,
    ty: &'a Type,
    attrs: &'a Vec<Attribute>,
}

impl<'a> MsgField<'a> {
    /// Creates new field from trait method argument
    pub fn new(item: &'a PatType, generics_checker: &mut CheckGenerics) -> Option<MsgField<'a>> {
        let name = match &*item.pat {
            Pat::Ident(p) => Some(&p.ident),
            pat => {
                // TODO: Support pattern arguments, when decorated with argument with item
                // name
                //
                // Eg.
                //
                // ```
                // fn exec_foo(&self, ctx: Ctx, #[msg(name=metadata)] SomeData { addr, sender }: SomeData);
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
        let attrs = &item.attrs;
        generics_checker.visit_type(ty);

        Some(Self { name, ty, attrs })
    }

    /// Emits message field
    pub fn emit(&self) -> TokenStream {
        let Self { name, ty, attrs } = self;

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #(#attrs)*
                #name: #ty
            }
        }
    }

    pub fn name(&self) -> &'a Ident {
        self.name
    }
}

/// Glue message is the message composing Exec/Query messages from several traits
#[derive(Debug)]
pub struct GlueMessage<'a> {
    interfaces: Vec<ContractMessageAttr>,
    name: &'a Ident,
    contract: &'a Type,
    msg_ty: MsgType,
    error: &'a Type,
}

impl<'a> GlueMessage<'a> {
    #[cfg(not(tarpaulin_include))]
    // Lack of coverage here is false negative due to usage in closures
    fn merge_module_with_name(module: &syn::Path, name: &syn::Ident) -> syn::Ident {
        let segments = &module.segments;
        assert!(!segments.is_empty());

        let syn::PathSegment { ident, .. } = &segments[0];
        let module_name = ident.to_string().to_case(Case::UpperCamel);
        syn::Ident::new(&format!("{}{}", module_name, name), name.span())
    }

    pub fn new(name: &'a Ident, source: &'a ItemImpl, msg_ty: MsgType, error: &'a Type) -> Self {
        let interfaces: Vec<_> = source
            .attrs
            .iter()
            .filter(|attr| attr.path.is_ident("messages"))
            .filter_map(|attr| {
                let interface = match ContractMessageAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(interface) => interface,
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        return None;
                    }
                };

                Some(interface)
            })
            .collect();

        GlueMessage {
            interfaces,
            name,
            contract: &source.self_ty,
            msg_ty,
            error,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();

        let Self {
            interfaces,
            name,
            contract,
            msg_ty,
            error,
        } = self;
        let contract = StripGenerics.fold_type((*contract).clone());
        let contract_name = Ident::new(&format!("Contract{}", name), name.span());

        let variants = interfaces.iter().map(|interface| {
            let ContractMessageAttr {
                module,
                exec_generic_params,
                query_generic_params,
                variant,
            } = interface;

            let generics = match msg_ty {
                MsgType::Exec => exec_generic_params.as_slice(),
                MsgType::Query => query_generic_params.as_slice(),
                _ => &[],
            };

            let enum_name = GlueMessage::merge_module_with_name(module, name);
            quote! { #variant(#module :: #enum_name<#(#generics,)*>) }
        });

        let msg_name = quote! {#contract ( #name)};
        let mut interface_names: Vec<TokenStream> = interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, .. } = interface;

                let enum_name = GlueMessage::merge_module_with_name(module, name);
                quote! { &#module :: #enum_name :: messages()}
            })
            .collect();
        interface_names.push(quote! {&#name :: messages()});

        let interfaces_cnt = interface_names.len();

        let dispatch_arms = interfaces.iter().map(|interface| {
            let ContractMessageAttr { variant, .. } = interface;

            quote! { #contract_name :: #variant(msg) => msg.dispatch(contract, Into::into(ctx)) }
        });

        let dispatch_arm = quote! {#contract_name :: #contract (msg) =>msg.dispatch(contract, ctx)};

        let deserialization_attempts = interfaces.iter().map(|interface| {
            let ContractMessageAttr {
                module, variant, ..
            } = interface;
            let enum_name = GlueMessage::merge_module_with_name(module, name);
            quote! {
                let msgs = &#module :: #enum_name ::messages();
                if msgs.into_iter().any(|msg| msg == &recv_msg_name) {
                    match val.deserialize_into() {
                        Ok(msg) => return Ok(Self:: #variant (msg)),
                        Err(err) => return Err(D::Error::custom(err)).map(Self:: #variant),
                    };
                }
            }
        });

        #[cfg(not(tarpaulin_include))]
        let contract_deserialization_attempt = quote! {
            let msgs = &#name :: messages();
            if msgs.into_iter().any(|msg| msg == &recv_msg_name) {
                match val.deserialize_into() {
                    Ok(msg) => return Ok(Self:: #contract (msg)),
                    Err(err) => return Err(D::Error::custom(err)).map(Self:: #contract)
                };
            }
        };

        let ctx_type = msg_ty.emit_ctx_type();
        let ret_type = msg_ty.emit_result_type(&None, error);

        let mut response_schemas: Vec<TokenStream> = interfaces
            .iter()
            .map(|interface| {
                let ContractMessageAttr { module, .. } = interface;

                let enum_name = GlueMessage::merge_module_with_name(module, name);
                quote! { #module :: #enum_name :: response_schemas_impl()}
            })
            .collect();
        response_schemas.push(quote! {#name :: response_schemas_impl()});

        let response_schemas = match name.to_string().as_str() {
            "QueryMsg" => {
                #[cfg(not(tarpaulin_include))]
                {
                    quote! {
                        #[cfg(not(target_arch = "wasm32"))]
                        impl cosmwasm_schema::QueryResponses for #contract_name {
                            fn response_schemas_impl() -> std::collections::BTreeMap<String, #sylvia ::schemars::schema::RootSchema> {
                                let responses = [#(#response_schemas),*];
                                responses.into_iter().flatten().collect()
                            }
                        }
                    }
                }
            }
            _ => {
                quote! {}
            }
        };

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                #[allow(clippy::derive_partial_eq_without_eq)]
                #[derive(#sylvia ::serde::Serialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                #[serde(rename_all="snake_case", untagged)]
                pub enum #contract_name {
                    #(#variants,)*
                    #msg_name
                }

                impl #contract_name {
                    pub fn dispatch(
                        self,
                        contract: &#contract,
                        ctx: #ctx_type,
                    ) -> #ret_type {
                        const _: () = {
                            let msgs: [&[&str]; #interfaces_cnt] = [#(#interface_names),*];
                            #sylvia ::utils::assert_no_intersection(msgs);
                        };

                        match self {
                            #(#dispatch_arms,)*
                            #dispatch_arm
                        }
                    }
                }

                #response_schemas

                impl<'de> serde::Deserialize<'de> for #contract_name {
                    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                        where D: serde::Deserializer<'de>,
                    {
                        use serde::de::Error;

                        let val = #sylvia ::serde_value::Value::deserialize(deserializer)?;
                        let map = match &val {
                            #sylvia ::serde_value::Value::Map(map) => map,
                            _ => panic!("Expected msg to be Value variant Map. Possibly an issue with msg format.")
                        };
                        if map.len() != 1 {
                            panic!("Found more or zero msgs after deserialization. Expected one.")
                        }
                        // Due to earlier size check of map this unwrap is safe
                        let recv_msg_name = map.into_iter().next().unwrap();

                        if let #sylvia ::serde_value::Value::String(recv_msg_name) = &recv_msg_name .0 {
                            #(#deserialization_attempts)*
                            #contract_deserialization_attempt
                        }

                        let msgs: [&[&str]; #interfaces_cnt] = [#(#interface_names),*];
                        let mut err_msg = msgs.into_iter().flatten().fold(
                            // It might be better to forward the error or serialization, but we just
                            // deserialized it from JSON, not reason to expect failure here.
                            format!(
                                "Unsupported message received: {}. Messages supported by this contract: ",
                                #sylvia ::serde_json::to_string(&val).unwrap_or_else(|_| String::new())
                            ),
                            |mut acc, message| acc + message + ", ",
                        );
                        err_msg.truncate(err_msg.len() - 2);
                        Err(D::Error::custom(err_msg))
                    }
                }
            }
        }
    }
}

pub struct EntryPoints {
    name: Type,
    error: Type,
}

impl EntryPoints {
    pub fn new(source: &ItemImpl) -> Self {
        let sylvia = crate_module();
        let name = StripGenerics.fold_type(*source.self_ty.clone());

        let error = source
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("error"))
            .and_then(
                |attr| match ContractErrorAttr::parse.parse2(attr.tokens.clone()) {
                    Ok(error) => Some(error.error),
                    Err(err) => {
                        emit_error!(attr.span(), err);
                        None
                    }
                },
            )
            .unwrap_or_else(|| parse_quote! { #sylvia ::cw_std::StdError });

        Self { name, error }
    }

    pub fn emit(&self) -> TokenStream {
        let Self { name, error } = self;
        let sylvia = crate_module();

        #[cfg(not(tarpaulin_include))]
        {
            quote! {
                pub mod entry_points {
                    use super::*;
                    const CONTRACT: #name = #name ::new();

                    #[#sylvia ::cw_std::entry_point]
                    pub fn instantiate(
                        deps: #sylvia ::cw_std::DepsMut,
                        env: #sylvia ::cw_std::Env,
                        info: #sylvia ::cw_std::MessageInfo,
                        msg: InstantiateMsg,
                    ) -> Result<#sylvia ::cw_std::Response, #error> {
                        msg.dispatch(&CONTRACT, (deps, env, info)).map_err(Into::into)
                    }

                    #[#sylvia ::cw_std::entry_point]
                    pub fn execute(
                        deps: #sylvia ::cw_std::DepsMut,
                        env: #sylvia ::cw_std::Env,
                        info: #sylvia ::cw_std::MessageInfo,
                        msg: ContractExecMsg,
                    ) -> Result<#sylvia ::cw_std::Response, #error> {
                        msg.dispatch(&CONTRACT, (deps, env, info)).map_err(Into::into)
                    }

                    #[#sylvia ::cw_std::entry_point]
                    pub fn query(deps: #sylvia ::cw_std::Deps, env: #sylvia ::cw_std::Env, msg: ContractQueryMsg) -> Result<#sylvia ::cw_std::Binary, #error> {
                        msg.dispatch(&CONTRACT, (deps, env)).map_err(Into::into)
                    }
                }
            }
        }
    }
}
