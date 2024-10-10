use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::{parse_quote, GenericParam, Ident, ItemImpl, Type};

use crate::crate_module;
use crate::parser::attributes::msg::ReplyOn;
use crate::parser::{MsgType, SylviaAttribute};
use crate::types::msg_field::MsgField;
use crate::types::msg_variant::{MsgVariant, MsgVariants};
use crate::utils::emit_turbofish;

pub struct Reply<'a> {
    source: &'a ItemImpl,
    generics: &'a [&'a GenericParam],
    reply_data: Vec<ReplyData<'a>>,
}

impl<'a> Reply<'a> {
    pub fn new(
        source: &'a ItemImpl,
        generics: &'a [&'a GenericParam],
        variants: &'a MsgVariants<'a, GenericParam>,
    ) -> Self {
        let reply_data = variants.as_reply_data();

        Self {
            source,
            generics,
            reply_data,
        }
    }

    pub fn emit(&self) -> TokenStream {
        let unique_handlers: Vec<_> = self.emit_reply_ids().collect();
        let dispatch = self.emit_dispatch();
        let sub_msg_trait = self.emit_sub_msg_trait();

        quote! {
            #(#unique_handlers)*

            #dispatch

            #sub_msg_trait
        }
    }

    /// Generates dispatch method that matches over every generated `ReplyId`
    /// and dispatches depending on the [`ReplyOn`](crate::parser::attributes::msg::ReplyOn).
    pub fn emit_dispatch(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            source,
            generics,
            reply_data,
            ..
        } = self;

        let msg_ty = MsgType::Reply;
        let contract = &source.self_ty;
        let where_clause = &source.generics.where_clause;

        let custom_query = parse_quote!( < #contract as #sylvia ::types::ContractApi>::CustomQuery);
        let custom_msg = parse_quote!( < #contract as #sylvia ::types::ContractApi>::CustomMsg);
        let error = parse_quote!( < #contract as #sylvia ::types::ContractApi>::Error);
        let ctx_params = msg_ty.emit_ctx_params(&custom_query);
        let ret_type = msg_ty.emit_result_type(&custom_msg, &error);

        let match_arms = reply_data
            .iter()
            .map(|data| data.emit_match_arms(contract, generics));

        quote! {
            pub fn dispatch_reply < #(#generics),* >( #ctx_params , msg: #sylvia ::cw_std::Reply, contract: #contract ) -> #ret_type #where_clause {
                let #sylvia ::cw_std::Reply {
                    id,
                    payload,
                    gas_used,
                    result,
                } = msg;

                match id {
                    #(#match_arms,)*
                    _ => {
                        let err_msg = format!("Unknown reply id: {}.", id);
                        Err( #sylvia ::cw_std::StdError::generic_err(err_msg)).map_err(Into::into)
                    }
                }
            }
        }
    }

    /// Generates `ReplyId`s for every unique
    /// [`reply_handler`](crate::parser::attributes::msg::MsgAttr::reply_handlers) and
    /// [`function_name`](crate::types::msg_variant::MsgVariant::function_name) not anotated with
    /// the `#[sv::msg(..)]` attribute with the `handlers` parameter.
    fn emit_reply_ids(&'a self) -> impl Iterator<Item = TokenStream> + 'a {
        self.reply_data.iter().enumerate().map(|(id, data)| {
            let id = id as u64;
            let reply_id = &data.reply_id;
            quote! {
                pub const #reply_id : u64 = #id ;
            }
        })
    }

    /// Generates `SubMsgMethods` trait with method for every reply id.
    fn emit_sub_msg_trait(&self) -> TokenStream {
        let Self { reply_data, .. } = self;

        let sylvia = crate_module();

        let methods_declaration = reply_data.iter().map(ReplyData::emit_submsg_trait_method);
        let submsg_reply_setters = reply_data.iter().map(ReplyData::emit_submsg_setter);
        let submsg_converters: Vec<_> = reply_data
            .iter()
            .map(ReplyData::emit_submsg_converter)
            .collect();

        quote! {
            pub trait SubMsgMethods<CustomMsgT> {
                #(#methods_declaration)*
            }

            impl<CustomMsgT> SubMsgMethods<CustomMsgT> for #sylvia ::cw_std::SubMsg<CustomMsgT> {
                #(#submsg_reply_setters)*
            }

            impl<CustomMsgT> SubMsgMethods<CustomMsgT> for #sylvia ::cw_std::WasmMsg {
                #(#submsg_converters)*
            }

            impl<CustomMsgT> SubMsgMethods<CustomMsgT> for #sylvia ::cw_std::CosmosMsg<CustomMsgT> {
                #(#submsg_converters)*
            }
        }
    }
}

trait ReplyVariants<'a> {
    /// Maps to [Vec<ReplyData>].
    /// Validation for duplicates and overlaps should be done in this method.
    fn as_reply_data(&self) -> Vec<ReplyData>;
}

impl<'a> ReplyVariants<'a> for MsgVariants<'a, GenericParam> {
    fn as_reply_data(&self) -> Vec<ReplyData> {
        let mut reply_data: Vec<ReplyData> = vec![];

        self.variants()
            .flat_map(ReplyVariant::as_variant_handlers_pair)
            .for_each(|(handler,  handler_id)| {
                let reply_on = handler.msg_attr().reply_on();
                let reply_id = handler_id.as_reply_id();
                match reply_data
                    .iter_mut()
                    .find(|existing_data| existing_data.reply_id == reply_id)
                {
                    Some(existing_data)
                        if existing_data
                            .handlers
                            .iter()
                            .any(|(_, existing_reply_on)| existing_reply_on.excludes(&reply_on)) =>
                    {
                        existing_data.handlers.iter().for_each(
                            |(existing_function_name, existing_reply_on)| {
                                let existing_reply_id = &existing_data.reply_id;

                                emit_error!(reply_id.span(), "Duplicated reply handler.";
                                    note = existing_data.reply_id.span() => format!("Previous definition of handler=`{}` for reply_on=`{}` defined on `fn {}()`", existing_reply_id, existing_reply_on, existing_function_name);
                                )
                            },
                        )
                    }
                    Some(existing_data) => existing_data.add_second_handler(handler),
                    None => reply_data.push(ReplyData::new(reply_id, handler,  handler_id)),
                }
            });

        reply_data
    }
}

/// Maps single reply id with its handlers.
struct ReplyData<'a> {
    /// Unique identifier for the reply.
    pub reply_id: Ident,
    /// Unique name of the handler from which the [reply_id](ReplyData::reply_id) was constructed.
    pub handler_id: &'a Ident,
    /// Methods handling the reply id for the associated reply on.
    pub handlers: Vec<(&'a Ident, ReplyOn)>,
    /// Payload parameters associated with the handlers.
    pub payload: Vec<&'a MsgField<'a>>,
}

impl<'a> ReplyData<'a> {
    pub fn new(reply_id: Ident, variant: &'a MsgVariant<'a>, handler_id: &'a Ident) -> Self {
        // Skip the first field reserved for the `data`.
        let payload = variant.fields().iter().skip(1).collect::<Vec<_>>();
        let method_name = variant.function_name();
        let reply_on = variant.msg_attr().reply_on();

        Self {
            reply_id,
            handler_id,
            handlers: vec![(method_name, reply_on)],
            payload,
        }
    }

    /// Adds second handler to the reply data provdided their payload signature match.
    pub fn add_second_handler(&mut self, new_handler: &'a MsgVariant<'a>) {
        let (current_method_name, _) = match self.handlers.first() {
            Some(handler) => handler,
            _ => return,
        };

        if self.payload.len() != new_handler.fields().len() - 1 {
            emit_error!(current_method_name.span(), "Mismatched lenght of method parameters.";
                note = self.handler_id.span() => format!("Both `{}` handlers should have the same number of parameters.", self.handler_id);
                note = new_handler.function_name().span() => format!("Previous definition of {} handler.", self.handler_id)
            );
        }

        self.payload
            .iter()
            .zip(new_handler.fields().iter().skip(1))
            .for_each(|(current_field, new_field)|
        {
            if current_field.ty() != new_field.ty() {
                emit_error!(current_field.name().span(), "Mismatched parameter in reply handlers.";
                    note = current_field.name().span() => format!("Parameters for the `{}` handler have to be the same.", self.handler_id);
                    note = new_field.name().span() => format!("Previous parameter defined for the `{}` handler.", self.handler_id)
                )
            }
        });

        let new_function_name = new_handler.function_name();
        let new_reply_on = new_handler.msg_attr().reply_on();
        self.handlers.push((new_function_name, new_reply_on));
    }

    /// Emits success and failure match arms for a single `ReplyId`.
    fn emit_match_arms(&self, contract: &Type, generics: &[&GenericParam]) -> TokenStream {
        let reply_id = &self.reply_id;
        let contract_turbofish = emit_turbofish(contract, generics);
        let success_match_arm = self.emit_success_match_arm(&contract_turbofish);
        let failure_match_arm = self.emit_failure_match_arm(&contract_turbofish);

        quote! {
            #reply_id => {
                match result {
                    #success_match_arm
                    #failure_match_arm
                }
            }
        }
    }

    /// Emits [cosmwasm_std::ReplyOn] value to be put in the `cosmwasm_std::SubMsg`.
    /// If both `Success` and `Failure` is defined for given `reply_id`, `cosmwasm_std::ReplyOn::Always` is returned.
    fn emit_cw_reply_on(&self) -> TokenStream {
        let sylvia = crate_module();
        let is_always = self
            .handlers
            .iter()
            .any(|(_, reply_on)| reply_on == &ReplyOn::Always);
        let is_success = self
            .handlers
            .iter()
            .any(|(_, reply_on)| reply_on == &ReplyOn::Success);
        let is_failure = self
            .handlers
            .iter()
            .any(|(_, reply_on)| reply_on == &ReplyOn::Failure);

        if is_always || (is_success && is_failure) {
            quote! { #sylvia ::cw_std::ReplyOn::Always }
        } else if is_success {
            quote! { #sylvia ::cw_std::ReplyOn::Success }
        } else if is_failure {
            quote! { #sylvia ::cw_std::ReplyOn::Error }
        } else {
            // This should never happen.
            // We parse only the `Success`, `Failure` and `Always` values which are covered above.
            // Handling the `Never` value wouldn't make sense as we would create a dead handler.
            quote! { #sylvia ::cw_std::ReplyOn::Never }
        }
    }

    /// Emits method setting reply related fields on the `cosmwasm_std::SubMsg`.
    fn emit_submsg_setter(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            reply_id,
            handler_id,
            payload,
            ..
        } = self;

        let method_name = handler_id;
        let reply_on = self.emit_cw_reply_on();
        let payload_parameters = payload.iter().map(|field| field.emit_method_field());
        let payload_serialization = payload.emit_payload_serialization();

        quote! {
            fn #method_name (self, #(#payload_parameters),* ) -> #sylvia ::cw_std::StdResult< #sylvia ::cw_std::SubMsg<CustomMsgT>> {
                #payload_serialization

                Ok( #sylvia ::cw_std::SubMsg {
                    reply_on: #reply_on ,
                    id: #reply_id ,
                    payload,
                    ..self
                })
            }
        }
    }

    /// Emits method for converting `WasmMsg` or `CosmosMsg` to `SubMsg`.
    fn emit_submsg_converter(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            reply_id,
            handler_id,
            payload,
            ..
        } = self;

        let method_name = handler_id;
        let reply_on = self.emit_cw_reply_on();
        let payload_parameters = payload.iter().map(|field| field.emit_method_field());
        let payload_serialization = payload.emit_payload_serialization();

        quote! {
            fn #method_name (self, #(#payload_parameters),* ) -> #sylvia ::cw_std::StdResult< #sylvia ::cw_std::SubMsg<CustomMsgT>> {
                #payload_serialization

                Ok( #sylvia ::cw_std::SubMsg {
                    reply_on: #reply_on ,
                    id: #reply_id ,
                    msg: self.into(),
                    payload,
                    gas_limit: None,
                })
            }
        }
    }

    fn emit_submsg_trait_method(&self) -> TokenStream {
        let sylvia = crate_module();
        let method_name = &self.handler_id;
        let payload_parameters = self.payload.iter().map(|field| field.emit_method_field());

        quote! {
            fn #method_name (self, #(#payload_parameters),* ) -> #sylvia ::cw_std::StdResult< #sylvia ::cw_std::SubMsg<CustomMsgT>>;
        }
    }

    /// Emits match arm for [ReplyOn::Success].
    /// In case neither [ReplyOn::Success] nor [ReplyOn::Always] is present, `Response::events`
    /// and `Response::data` are forwarded in the `Response`
    fn emit_success_match_arm(&self, contract_turbofish: &Type) -> TokenStream {
        let sylvia = crate_module();

        match self
            .handlers
            .iter()
            .find(|(_, reply_on)| reply_on == &ReplyOn::Success || reply_on == &ReplyOn::Always)
        {
            Some((method_name, reply_on)) if reply_on == &ReplyOn::Success => {
                let payload_values = self.payload.iter().map(|field| field.name());
                let payload_deserialization = self.payload.emit_payload_deserialization();

                quote! {
                    #sylvia ::cw_std::SubMsgResult::Ok(sub_msg_resp) => {
                        #[allow(deprecated)]
                        let #sylvia ::cw_std::SubMsgResponse { events, data, msg_responses} = sub_msg_resp;
                        #payload_deserialization

                        #contract_turbofish ::new(). #method_name ((deps, env, gas_used, events, msg_responses).into(), data, #(#payload_values),* )
                    }
                }
            }
            Some((method_name, reply_on)) if reply_on == &ReplyOn::Always => {
                let payload_values = self.payload.iter().map(|field| field.name());
                let payload_deserialization = self.payload.emit_payload_deserialization();

                quote! {
                    #sylvia ::cw_std::SubMsgResult::Ok(_) => {
                        #payload_deserialization

                        #contract_turbofish ::new(). #method_name ((deps, env, gas_used, vec![], vec![]).into(), result, #(#payload_values),* )
                    }
                }
            }
            _ => quote! {
                #sylvia ::cw_std::SubMsgResult::Ok(sub_msg_resp) => {
                    let mut resp = sylvia::cw_std::Response::new().add_events(sub_msg_resp.events);

                    #[allow(deprecated)]
                    if sub_msg_resp.data.is_some() {
                        resp = resp.set_data(sub_msg_resp.data.unwrap());
                    }

                    Ok(resp)
                }
            },
        }
    }

    /// Emits match arm for [ReplyOn::Failure].
    /// In case neither [ReplyOn::Failure] nor [ReplyOn::Always] is present,
    /// the error is forwarded.
    fn emit_failure_match_arm(&self, contract_turbofish: &Type) -> TokenStream {
        let sylvia = crate_module();

        match self
            .handlers
            .iter()
            .find(|(_, reply_on)| reply_on == &ReplyOn::Failure || reply_on == &ReplyOn::Always)
        {
            Some((method_name, reply_on)) if reply_on == &ReplyOn::Failure => {
                let payload_values = self.payload.iter().map(|field| field.name());
                let payload_deserialization = self.payload.emit_payload_deserialization();

                quote! {
                    #sylvia ::cw_std::SubMsgResult::Err(error) => {
                        #payload_deserialization

                        #contract_turbofish ::new(). #method_name ((deps, env, gas_used, vec![], vec![]).into(), error, #(#payload_values),* )
                    }
                }
            }
            Some((method_name, reply_on)) if reply_on == &ReplyOn::Always => {
                let payload_values = self.payload.iter().map(|field| field.name());
                let payload_deserialization = self.payload.emit_payload_deserialization();

                quote! {
                    #sylvia ::cw_std::SubMsgResult::Err(_) => {
                        #payload_deserialization

                        #contract_turbofish ::new(). #method_name ((deps, env, gas_used, vec![], vec![]).into(), result, #(#payload_values),* )
                    }
                }
            }
            _ => quote! {
                #sylvia ::cw_std::SubMsgResult::Err(error) => {
                    Err(sylvia::cw_std::StdError::generic_err(error)).map_err(Into::into)
                }
            },
        }
    }
}

trait ReplyVariant<'a> {
    fn as_variant_handlers_pair(&'a self) -> Vec<(&'a MsgVariant<'a>, &'a Ident)>;
}

impl<'a> ReplyVariant<'a> for MsgVariant<'a> {
    fn as_variant_handlers_pair(&'a self) -> Vec<(&'a MsgVariant<'a>, &'a Ident)> {
        let variant_handler_id_pair: Vec<_> = self
            .msg_attr()
            .handlers()
            .iter()
            .map(|handler| (self, handler))
            .collect();

        if variant_handler_id_pair.is_empty() {
            return vec![(self, self.function_name())];
        }

        variant_handler_id_pair
    }
}

pub trait PayloadFields {
    fn emit_payload_deserialization(&self) -> TokenStream;
    fn emit_payload_serialization(&self) -> TokenStream;
    fn is_payload_marked(&self) -> bool;
}

impl PayloadFields for Vec<&MsgField<'_>> {
    fn emit_payload_deserialization(&self) -> TokenStream {
        let sylvia = crate_module();
        if self.is_payload_marked() {
            // Safe to unwrap as we check if the payload exist.
            let payload_value = self.first().unwrap().name();
            return quote! {
                let #payload_value = payload ;
            };
        }

        let deserialized_payload_names = self.iter().map(|field| field.name());
        quote! {
            let ( #(#deserialized_payload_names),* ) = #sylvia ::cw_std::from_json(&payload)?;
        }
    }

    fn emit_payload_serialization(&self) -> TokenStream {
        let sylvia = crate_module();
        if self.is_payload_marked() {
            // Safe to unwrap as we check if the payload exist.
            let payload_value = self.first().unwrap().name();
            return quote! {
                let payload = #payload_value ;
            };
        }

        let payload_values = self.iter().map(|field| field.name());
        quote! {
            let payload = #sylvia ::cw_std::to_json_binary(&( #(#payload_values),* ))?;
        }
    }

    fn is_payload_marked(&self) -> bool {
        self.iter()
            .any(|field| field.contains_attribute(SylviaAttribute::Payload))
    }
}

/// Maps self to an [Ident] reply id.
trait AsReplyId {
    fn as_reply_id(&self) -> Ident;
}

impl AsReplyId for Ident {
    fn as_reply_id(&self) -> Ident {
        let reply_id = format! {"{}_REPLY_ID", self.to_string().to_case(Case::UpperSnake)};
        Ident::new(&reply_id, self.span())
    }
}
