//! Module responsible for generating `Reply` related code.
//!
//! Based on methods marked with the `#[sv::msg(reply)]` attribute, this module generates:
//!     - reply ids for every unique handler,
//!     - dispatch method that matches over every generated `ReplyId` and dispatches depending on the `ReplyOn`,
//!     - `SubMsgMethods` trait with method for every reply id.

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::{parse_quote, GenericParam, Ident, ItemImpl, Type};

use crate::crate_module;
use crate::parser::attributes::msg::ReplyOn;
use crate::parser::{MsgType, ParsedSylviaAttributes};
use crate::types::msg_field::MsgField;
use crate::types::msg_variant::{MsgVariant, MsgVariants};
use crate::utils::emit_turbofish;

const NUMBER_OF_ALLOWED_DATA_FIELDS: usize = 1;
const NUMBER_OF_ALLOWED_RAW_PAYLOAD_FIELDS: usize = 1;

/// Make sure that there are no additional parameters between ones marked
/// with `sv::data` and `sv::payload` and after the one marked with `sv::payload`.
fn assert_no_redundant_params(payload: &[&MsgField]) {
    let payload_param = payload.iter().enumerate().find(|(_, field)| {
        ParsedSylviaAttributes::new(field.attrs().iter())
            .payload
            .is_some()
    });

    if payload.len() == NUMBER_OF_ALLOWED_RAW_PAYLOAD_FIELDS {
        return;
    }

    let Some((index, payload_param)) = payload_param else {
        return;
    };

    if index == 0 {
        emit_error!(payload[1].name().span(), "Redundant payload parameter.";
            note = payload_param.name().span() => "Expected no parameters after the parameter marked with `#[sv::payload(raw)]`."
        )
    } else {
        emit_error!(payload[0].name().span(), "Redundant payload parameter.";
            note = payload_param.name().span() => "Expected no parameters between the parameter marked with `#[sv::data]` and `#[sv::payload(raw)]`."
        )
    }
}

pub struct Reply<'a> {
    source: &'a ItemImpl,
    generics: &'a [&'a GenericParam],
    reply_data: Vec<ReplyData<'a>>,
    error: Type,
}

impl<'a> Reply<'a> {
    pub fn new(
        source: &'a ItemImpl,
        generics: &'a [&'a GenericParam],
        variants: &'a MsgVariants<'a, GenericParam>,
    ) -> Self {
        let reply_data = variants.as_reply_data();
        let parsed_attrs = ParsedSylviaAttributes::new(source.attrs.iter());
        let error = parsed_attrs.error_attrs.unwrap_or_default().error;

        Self {
            source,
            generics,
            reply_data,
            error,
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
            error,
            ..
        } = self;

        let msg_ty = MsgType::Reply;
        let contract = &source.self_ty;
        let where_clause = &source.generics.where_clause;

        let custom_query = parse_quote!( < #contract as #sylvia ::types::ContractApi>::CustomQuery);
        let custom_msg = parse_quote!( < #contract as #sylvia ::types::ContractApi>::CustomMsg);
        let ctx_params = msg_ty.emit_ctx_params(&custom_query);
        let ret_type = msg_ty.emit_result_type(&custom_msg, error);

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
                    Some(existing_data) => existing_data.merge(handler),
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
    /// Data parameter associated with the handlers.
    pub data: Option<&'a MsgField<'a>>,
    /// Payload parameters associated with the handlers.
    pub payload: Vec<&'a MsgField<'a>>,
}

impl<'a> ReplyData<'a> {
    pub fn new(reply_id: Ident, variant: &'a MsgVariant<'a>, handler_id: &'a Ident) -> Self {
        let data = variant.as_data_field();
        variant.validate_fields_attributes();
        let payload = variant.fields().iter();
        let payload = if data.is_some() || variant.msg_attr().reply_on() != ReplyOn::Success {
            payload
                .skip(NUMBER_OF_ALLOWED_DATA_FIELDS)
                .collect::<Vec<_>>()
        } else {
            payload.collect::<Vec<_>>()
        };

        if payload.is_empty() {
            emit_error!(variant.name().span(), "Missing payload parameter.";
                note =  "Expected at least one payload parameter at the end of parameter list."
            )
        }

        assert_no_redundant_params(&payload);
        let method_name = variant.function_name();
        let reply_on = variant.msg_attr().reply_on();

        Self {
            reply_id,
            handler_id,
            handlers: vec![(method_name, reply_on)],
            data,
            payload,
        }
    }

    /// Adds second handler to the reply data provdided their payload signature match.
    pub fn merge(&mut self, new_handler: &'a MsgVariant<'a>) {
        let (current_method_name, _) = match self.handlers.first() {
            Some(handler) => handler,
            _ => return,
        };

        let new_reply_data = ReplyData::new(self.reply_id.clone(), new_handler, self.handler_id);

        if self.payload.len() != new_reply_data.payload.len() {
            emit_error!(current_method_name.span(), "Mismatched quantity of method parameters.";
                note = self.handler_id.span() => format!("Both `{}` handlers should have the same number of parameters.", self.handler_id);
                note = new_handler.function_name().span() => format!("Previous definition of {} handler.", self.handler_id)
            );
        }

        self.payload
            .iter()
            .zip(new_reply_data.payload.iter())
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

    /// Emits success and error match arms for a single `ReplyId`.
    fn emit_match_arms(&self, contract: &Type, generics: &[&GenericParam]) -> TokenStream {
        let reply_id = &self.reply_id;
        let contract_turbofish = emit_turbofish(contract, generics);
        let success_match_arm = self.emit_success_match_arm(&contract_turbofish);
        let error_match_arm = self.emit_error_match_arm(&contract_turbofish);

        quote! {
            #reply_id => {
                match result {
                    #success_match_arm
                    #error_match_arm
                }
            }
        }
    }

    /// Emits [cosmwasm_std::ReplyOn] value to be put in the `cosmwasm_std::SubMsg`.
    /// If both `Success` and `Error` is defined for given `reply_id`, `cosmwasm_std::ReplyOn::Always` is returned.
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
        let is_error = self
            .handlers
            .iter()
            .any(|(_, reply_on)| reply_on == &ReplyOn::Error);

        if is_always || (is_success && is_error) {
            quote! { #sylvia ::cw_std::ReplyOn::Always }
        } else if is_success {
            quote! { #sylvia ::cw_std::ReplyOn::Success }
        } else {
            quote! { #sylvia ::cw_std::ReplyOn::Error }
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
                let data_deserialization = self.data.map(DataField::emit_data_deserialization);
                let data = self.data.map(|_| quote! { data, });

                quote! {
                    #sylvia ::cw_std::SubMsgResult::Ok(sub_msg_resp) => {
                        #[allow(deprecated)]
                        let #sylvia ::cw_std::SubMsgResponse { events, data, msg_responses} = sub_msg_resp;
                        #payload_deserialization
                        #data_deserialization

                        #contract_turbofish ::new(). #method_name ((deps, env, gas_used, events, msg_responses).into(), #data #(#payload_values),* )
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

    /// Emits match arm for [ReplyOn::Error].
    /// In case neither [ReplyOn::Error] nor [ReplyOn::Always] is present,
    /// the error is forwarded.
    fn emit_error_match_arm(&self, contract_turbofish: &Type) -> TokenStream {
        let sylvia = crate_module();

        match self
            .handlers
            .iter()
            .find(|(_, reply_on)| reply_on == &ReplyOn::Error || reply_on == &ReplyOn::Always)
        {
            Some((method_name, reply_on)) if reply_on == &ReplyOn::Error => {
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
    fn as_data_field(&'a self) -> Option<&'a MsgField<'a>>;
    fn validate_fields_attributes(&'a self);
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

    /// Validates attributes and returns `Some(MsgField)` if a field marked with `sv::data` attribute
    /// is present and the `reply_on` attribute is set to `ReplyOn::Success`.
    fn as_data_field(&'a self) -> Option<&'a MsgField<'a>> {
        let data_param = self.fields().iter().enumerate().find(|(_, field)| {
            ParsedSylviaAttributes::new(field.attrs().iter())
                .data
                .is_some()
        });
        match data_param {
            Some((index, field))
                if self.msg_attr().reply_on() == ReplyOn::Success && index == 0 =>
            {
                Some(field)
            }
            Some((index, field))
                if self.msg_attr().reply_on() == ReplyOn::Success && index != 0 =>
            {
                emit_error!(field.name().span(), "Wrong usage of `#[sv::data]` attribute.";
                    note = "The `#[sv::data]` attribute can only be used on the first parameter after the `ReplyCtx`."
                );
                None
            }
            Some((_, field)) if self.msg_attr().reply_on() != ReplyOn::Success => {
                emit_error!(field.name().span(), "Wrong usage of `#[sv::data]` attribute.";
                    note = "The `#[sv::data]` attribute can only be used in `success` scenario.";
                    note = format!("Found usage in `{}` scenario.", self.msg_attr().reply_on())
                );
                None
            }
            _ => None,
        }
    }

    /// Validates if the fields attributes are correct.
    fn validate_fields_attributes(&'a self) {
        let field_attrs = self.fields().iter().flat_map(|field| field.attrs());
        ParsedSylviaAttributes::new(field_attrs);
    }
}

pub trait DataField {
    fn emit_data_deserialization(&self) -> TokenStream;
}

impl DataField for MsgField<'_> {
    fn emit_data_deserialization(&self) -> TokenStream {
        let sylvia = crate_module();
        let data = ParsedSylviaAttributes::new(self.attrs().iter()).data;
        let missing_data_err = "Missing reply data field.";
        let transaction_id = quote! {
            env
                .transaction
                .as_ref()
                .map(|tx| format!("{}", &tx.index))
                .unwrap_or_else(|| "Missing".to_string())
        };
        let invalid_reply_data_err = quote! {
            format! {"Invalid reply data at block height: {}, transaction id: {}.\nSerde error while deserializing {}",
                env.block.height,
                #transaction_id,
                err}
        };
        let execute_data_deserialization = quote! {
            let deserialized_data =
                #sylvia ::cw_utils::parse_execute_response_data(data.as_slice())
                    .map_err(|err| #sylvia ::cw_std::StdError::generic_err(
                        format!("Failed deserializing protobuf data: {}", err)
                    ))?;
            let deserialized_data = match deserialized_data.data {
                Some(data) => #sylvia ::cw_std::from_json(&data).map_err(|err| #sylvia ::cw_std::StdError::generic_err( #invalid_reply_data_err ))?,
                None => return Err(Into::into( #sylvia ::cw_std::StdError::generic_err( #missing_data_err ))),
            };
        };

        let instantiate_data_deserialization = quote! {
            let deserialized_data =
                #sylvia ::cw_utils::parse_instantiate_response_data(data.as_slice())
                    .map_err(|err| #sylvia ::cw_std::StdError::generic_err(
                        format!("Failed deserializing protobuf data: {}", err)
                    ))?;
        };

        match data {
            Some(data) if data.raw && data.opt => quote! {},
            Some(data) if data.raw => quote! {
                let data = match data {
                    Some(data) => data,
                    None => return Err(Into::into( #sylvia ::cw_std::StdError::generic_err( #missing_data_err ))),
                };
            },
            Some(data) if data.instantiate && data.opt => quote! {
                let data = match data {
                    Some(data) => {
                        #instantiate_data_deserialization

                        Some(deserialized_data)
                    },
                    None => None,
                };
            },
            Some(data) if data.instantiate => quote! {
                let data = match data {
                    Some(data) => {
                        #instantiate_data_deserialization

                        deserialized_data
                    },
                    None => return Err(Into::into( #sylvia ::cw_std::StdError::generic_err( #missing_data_err ))),
                };
            },
            Some(data) if data.opt => quote! {
                let data = match data {
                    Some(data) => {
                        #execute_data_deserialization

                        Some(deserialized_data)
                    },
                    None => None,
                };
            },
            _ => quote! {
                let data = match data {
                    Some(data) => {
                        #execute_data_deserialization

                        deserialized_data
                    },
                    None => return Err(Into::into( #sylvia ::cw_std::StdError::generic_err( #missing_data_err ))),
                };
            },
        }
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
        self.iter().any(|field| {
            ParsedSylviaAttributes::new(field.attrs().iter())
                .payload
                .is_some()
        })
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
