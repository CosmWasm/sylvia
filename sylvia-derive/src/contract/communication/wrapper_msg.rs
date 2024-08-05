use crate::crate_module;
use crate::fold::StripGenerics;
use crate::parser::{ContractErrorAttr, Custom, MsgType};
use crate::types::interfaces::Interfaces;
use crate::utils::emit_bracketed_generics;
use proc_macro2::TokenStream;
use quote::quote;
use syn::fold::Fold;
use syn::spanned::Spanned;
use syn::{Ident, ItemImpl, Type};

/// Glue message is the message composing Exec/Query messages from several traits
#[derive(Debug)]
pub struct GlueMessage<'a> {
    source: &'a ItemImpl,
    contract: &'a Type,
    msg_ty: MsgType,
    error: &'a ContractErrorAttr,
    custom: &'a Custom,
    interfaces: &'a Interfaces,
}

impl<'a> GlueMessage<'a> {
    pub fn new(
        source: &'a ItemImpl,
        msg_ty: MsgType,
        error: &'a ContractErrorAttr,
        custom: &'a Custom,
        interfaces: &'a Interfaces,
    ) -> Self {
        GlueMessage {
            source,
            contract: &source.self_ty,
            msg_ty,
            error,
            custom,
            interfaces,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            contract,
            msg_ty,
            error,
            custom,
            interfaces,
            ..
        } = self;

        let generics: Vec<_> = source.generics.params.iter().collect();
        let full_where_clause = &source.generics.where_clause;
        let bracketed_wrapper_generics = emit_bracketed_generics(&generics);

        let contract_enum_name = msg_ty.emit_msg_wrapper_name();
        let enum_accessor = msg_ty.as_accessor_name();
        let contract_name = StripGenerics.fold_type((*contract).clone());

        let variants = interfaces.emit_glue_message_variants(msg_ty, contract);
        let types = interfaces.emit_glue_message_types(msg_ty, contract);

        let ep_name = msg_ty.emit_ep_name();
        let messages_fn_name = Ident::new(&format!("{}_messages", ep_name), contract.span());
        let contract_variant = quote! { #contract_name ( <#contract as #sylvia ::types::ContractApi> :: #enum_accessor ) };
        let mut messages_call = interfaces.emit_messages_call(msg_ty);
        messages_call.push(quote! { &#messages_fn_name() });

        let variants_cnt = messages_call.len();

        let dispatch_arms = interfaces.emit_dispatch_arms(msg_ty);

        let dispatch_arm =
            quote! {#contract_enum_name :: #contract_name (msg) => msg.dispatch(contract, ctx)};

        let interfaces_deserialization_attempts = interfaces.emit_deserialization_attempts(msg_ty);

        let contract_deserialization_attempt = quote! {
            let msgs = &#messages_fn_name();
            if msgs.into_iter().any(|msg| msg == &recv_msg_name) {
                match val.deserialize_into() {
                    Ok(msg) => return Ok(Self:: #contract_name (msg)),
                    Err(err) => return Err(D::Error::custom(err)).map(Self:: #contract_name )
                };
            }
        };

        let ctx_type = msg_ty.emit_ctx_type(&custom.query_or_default());
        let ret_type = msg_ty.emit_result_type(&custom.msg_or_default(), &error.error);

        let mut response_schemas_calls = interfaces.emit_response_schemas_calls(msg_ty, contract);
        response_schemas_calls
            .push(quote! {<#contract as #sylvia ::types::ContractApi> :: #enum_accessor ::response_schemas_impl()});

        let response_schemas = match msg_ty {
            MsgType::Query => {
                quote! {
                    #[cfg(not(target_arch = "wasm32"))]
                    impl #bracketed_wrapper_generics #sylvia ::cw_schema::QueryResponses for #contract_enum_name #bracketed_wrapper_generics #full_where_clause {
                        fn response_schemas_impl() -> std::collections::BTreeMap<String, #sylvia ::schemars::schema::RootSchema> {
                            let responses = [#(#response_schemas_calls),*];
                            responses.into_iter().flatten().collect()
                        }
                    }
                }
            }
            _ => {
                quote! {}
            }
        };

        let modules_names = interfaces.variants_modules();
        let variants_names = interfaces.variants_names();

        quote! {
            #[allow(clippy::derive_partial_eq_without_eq)]
            #[derive(#sylvia ::serde::Serialize, Clone, Debug, PartialEq)]
            #[serde(rename_all="snake_case", untagged)]
            pub enum #contract_enum_name #bracketed_wrapper_generics #full_where_clause {
                #(#variants,)*
                #contract_variant
            }

            // `schemars` v0.8.16 requires every generic type to implement JsonSchema in
            // order to use derive JsonSchema macro. The goal of that trait bound is to
            // generate schema_name. Currently there's no way to provide such a name in an
            // attribute, so Sylvia needs to implement this trait manually:
            //
            impl #bracketed_wrapper_generics #sylvia ::schemars::JsonSchema
                for #contract_enum_name #bracketed_wrapper_generics #full_where_clause {

                fn schema_name() -> std::string::String {
                    {
                        let res = format!(
                                "{0}",
                                std::any::type_name::<Self>()
                        );
                        res
                    }
                }

                fn json_schema(
                    gen: &mut #sylvia ::schemars::gen::SchemaGenerator,
                ) -> #sylvia ::schemars::schema::Schema {
                    #sylvia ::schemars::schema::Schema::Object( #sylvia ::schemars::schema::SchemaObject {
                        subschemas: Some(
                            Box::new( #sylvia ::schemars::schema::SubschemaValidation {
                                any_of: Some(
                                    <[_]>::into_vec(
                                        Box::new([
                                            #(gen.subschema_for::<#types>(),)*
                                            gen.subschema_for::< <#contract as #sylvia ::types::ContractApi> :: #enum_accessor >(),
                                        ]),
                                    ),
                                ),
                                ..Default::default()
                            }),
                        ),
                        ..Default::default()
                    })
                }
            }

            impl #bracketed_wrapper_generics #contract_enum_name #bracketed_wrapper_generics #full_where_clause {
                pub fn dispatch (
                    self,
                    contract: &#contract,
                    ctx: #ctx_type,
                ) -> #ret_type #full_where_clause {
                    const _: () = {
                        let msgs: [&[&str]; #variants_cnt] = [#(#messages_call),*];
                        #sylvia ::utils::assert_no_intersection(msgs);
                    };

                    match self {
                        #(#dispatch_arms,)*
                        #dispatch_arm
                    }
                }
            }

            #response_schemas

            impl<'sv_de, #(#generics,)* > serde::Deserialize<'sv_de> for #contract_enum_name #bracketed_wrapper_generics #full_where_clause {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                    where D: serde::Deserializer<'sv_de>,
                {
                    use serde::de::Error;

                    let val = #sylvia ::serde_value::Value::deserialize(deserializer)?;
                    let map = match &val {
                        #sylvia ::serde_value::Value::Map(map) => map,
                        _ => return Err(D::Error::custom("Wrong message format!"))
                    };
                    if map.len() != 1 {
                        return Err(D::Error::custom(format!("Expected exactly one message. Received {}", map.len())))
                    }

                    // Due to earlier size check of map this unwrap is safe
                    let recv_msg_name = map.into_iter().next().unwrap();

                    if let #sylvia ::serde_value::Value::String(recv_msg_name) = &recv_msg_name .0 {
                        #(#interfaces_deserialization_attempts)*
                        #contract_deserialization_attempt
                    }

                    let msgs: [&[&str]; #variants_cnt] = [#(#messages_call),*];
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

            impl #bracketed_wrapper_generics From<<#contract as #sylvia ::types::ContractApi>:: #enum_accessor>
                for #contract_enum_name #bracketed_wrapper_generics #full_where_clause {
                fn from(a: <#contract as #sylvia ::types::ContractApi>:: #enum_accessor ) -> Self {
                    Self:: #contract_name (a)
                }
            }

            #(
            impl #bracketed_wrapper_generics From<<#contract as #modules_names ::sv::InterfaceMessagesApi>:: #enum_accessor>
                for #contract_enum_name #bracketed_wrapper_generics #full_where_clause {
                fn from(a: <#contract as #modules_names ::sv::InterfaceMessagesApi>:: #enum_accessor ) -> Self {
                    Self:: #variants_names (a)
                }
            }
            )*
        }
    }
}
