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

        let methods_declaration = reply_data.iter().map(|data| {
            let method_name = &data.handler_name;

            quote! {
                fn #method_name (self) -> #sylvia ::cw_std::SubMsg<CustomMsgT>;
            }
        });

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
            .flat_map(ReplyVariant::as_reply_data)
            .for_each(|(reply_id,  handler_name, handler)| {
                let reply_on = handler.msg_attr().reply_on();
                match reply_data
                    .iter_mut()
                    .find(|existing_data| existing_data.reply_id == reply_id)
                {
                    Some(existing_data)
                        if existing_data
                            .handlers
                            .iter()
                            .any(|existing_handler| existing_handler.msg_attr().reply_on().excludes(&reply_on)) =>
                    {
                        existing_data.handlers.iter().for_each(
                            |existing_handler| {
                                let existing_reply_id = &existing_data.reply_id;
                                let existing_reply_on = existing_handler.msg_attr().reply_on();
                                let existing_function_name= existing_handler.function_name();

                                emit_error!(reply_id.span(), "Duplicated reply handler.";
                                    note = existing_data.reply_id.span() => format!("Previous definition of handler={} for reply_on={} defined on `fn {}()`", existing_reply_id, existing_reply_on, existing_function_name);
                                )
                            },
                        )
                    }
                    Some(existing_data) => existing_data.handlers.push(handler),
                    None => reply_data.push(ReplyData::new(reply_id, handler,  handler_name)),
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
    pub handler_name: &'a Ident,
    /// Handler methods for the reply id.
    pub handlers: Vec<&'a MsgVariant<'a>>,
}

impl<'a> ReplyData<'a> {
    pub fn new(reply_id: Ident, variant: &'a MsgVariant<'a>, handler_name: &'a Ident) -> Self {
        Self {
            reply_id,
            handler_name,
            handlers: vec![variant],
        }
    }

    /// Emits success and failure match arms for a single `ReplyId`.
    fn emit_match_arms(&self, contract: &Type, generics: &[&GenericParam]) -> TokenStream {
        let Self {
            reply_id, handlers, ..
        } = self;

        let contract_turbofish = emit_turbofish(contract, generics);
        let success_match_arm = emit_success_match_arm(handlers, &contract_turbofish);
        let failure_match_arm = emit_failure_match_arm(handlers, &contract_turbofish);

        quote! {
            #reply_id => {
                match result {
                    #success_match_arm
                    #failure_match_arm
                }
            }
        }
    }

    fn emit_cw_reply_on(&self) -> TokenStream {
        let sylvia = crate_module();
        let is_always = self
            .handlers
            .iter()
            .any(|handler| handler.msg_attr().reply_on() == ReplyOn::Always);
        let is_success = self
            .handlers
            .iter()
            .any(|handler| handler.msg_attr().reply_on() == ReplyOn::Success);
        let is_failure = self
            .handlers
            .iter()
            .any(|handler| handler.msg_attr().reply_on() == ReplyOn::Failure);

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

    fn emit_submsg_setter(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            reply_id,
            handler_name,
            ..
        } = self;

        let method_name = handler_name;
        let reply_on = self.emit_cw_reply_on();

        quote! {
            fn #method_name (self) -> #sylvia ::cw_std::SubMsg<CustomMsgT> {
                #sylvia ::cw_std::SubMsg {
                    reply_on: #reply_on ,
                    id: #reply_id ,
                    ..self
                }
            }
        }
    }

    fn emit_submsg_converter(&self) -> TokenStream {
        let sylvia = crate_module();
        let Self {
            reply_id,
            handler_name,
            ..
        } = self;

        let method_name = handler_name;
        let reply_on = self.emit_cw_reply_on();

        quote! {
            fn #method_name (self) -> #sylvia ::cw_std::SubMsg<CustomMsgT> {
                #sylvia ::cw_std::SubMsg {
                    reply_on: #reply_on ,
                    id: #reply_id ,
                    msg: self.into(),
                    payload: Default::default(),
                    gas_limit: None,
                }
            }
        }
    }
}

/// Emits match arm for [ReplyOn::Success].
/// In case neither [ReplyOn::Success] nor [ReplyOn::Always] is present, `Response::events`
/// and `Response::data` are forwarded in the `Response`
fn emit_success_match_arm(handlers: &[&MsgVariant], contract_turbofish: &Type) -> TokenStream {
    let sylvia = crate_module();

    match handlers.iter().find(|handler| {
        handler.msg_attr().reply_on() == ReplyOn::Success
            || handler.msg_attr().reply_on() == ReplyOn::Always
    }) {
        Some(handler) if handler.msg_attr().reply_on() == ReplyOn::Success => {
            let function_name = handler.function_name();
            let payload = handler.emit_payload_parameters();
            let payload_deserialization = handler.emit_payload_deserialization();

            quote! {
                #sylvia ::cw_std::SubMsgResult::Ok(sub_msg_resp) => {
                    #[allow(deprecated)]
                    let #sylvia ::cw_std::SubMsgResponse { events, data, msg_responses} = sub_msg_resp;
                    #payload_deserialization

                    #contract_turbofish ::new(). #function_name ((deps, env, gas_used, events, msg_responses).into(), data, #payload )
                }
            }
        }
        Some(handler) if handler.msg_attr().reply_on() == ReplyOn::Always => {
            let function_name = handler.function_name();
            let payload = handler.emit_payload_parameters();
            let payload_deserialization = handler.emit_payload_deserialization();

            quote! {
                #sylvia ::cw_std::SubMsgResult::Ok(_) => {
                    #payload_deserialization

                    #contract_turbofish ::new(). #function_name ((deps, env, gas_used, vec![], vec![]).into(), result, #payload )
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
fn emit_failure_match_arm(handlers: &[&MsgVariant], contract_turbofish: &Type) -> TokenStream {
    let sylvia = crate_module();

    match handlers.iter().find(|handler| {
        handler.msg_attr().reply_on() == ReplyOn::Failure
            || handler.msg_attr().reply_on() == ReplyOn::Always
    }) {
        Some(handler) if handler.msg_attr().reply_on() == ReplyOn::Failure => {
            let function_name = handler.function_name();
            let payload = handler.emit_payload_parameters();
            let payload_deserialization = handler.emit_payload_deserialization();

            quote! {
                #sylvia ::cw_std::SubMsgResult::Err(error) => {
                    #payload_deserialization

                    #contract_turbofish ::new(). #function_name ((deps, env, gas_used, vec![], vec![]).into(), error, #payload )
                }
            }
        }
        Some(handler) if handler.msg_attr().reply_on() == ReplyOn::Always => {
            let function_name = handler.function_name();
            let payload = handler.emit_payload_parameters();
            let payload_deserialization = handler.emit_payload_deserialization();

            quote! {
                #sylvia ::cw_std::SubMsgResult::Err(_) => {
                    #payload_deserialization

                    #contract_turbofish ::new(). #function_name ((deps, env, gas_used, vec![], vec![]).into(), result, #payload )
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

trait ReplyVariant<'a> {
    fn as_handlers(&'a self) -> Vec<&'a Ident>;
    fn as_reply_data(&self) -> Vec<(Ident, &Ident, &MsgVariant)>;
    fn emit_payload_parameters(&self) -> TokenStream;
    fn emit_payload_deserialization(&self) -> TokenStream;
}

impl<'a> ReplyVariant<'a> for MsgVariant<'a> {
    fn as_handlers(&'a self) -> Vec<&'a Ident> {
        if self.msg_attr().handlers().is_empty() {
            return vec![self.function_name()];
        }
        self.msg_attr().handlers().iter().collect()
    }

    fn as_reply_data(&self) -> Vec<(Ident, &Ident, &MsgVariant)> {
        self.as_handlers()
            .iter()
            .map(|&handler| (handler.as_reply_id(), handler, self))
            .collect()
    }

    fn emit_payload_parameters(&self) -> TokenStream {
        if self
            .fields()
            .iter()
            .any(|field| field.contains_attribute(SylviaAttribute::Payload))
        {
            quote! { payload }
        } else {
            let deserialized_payload = self.fields().iter().skip(1).map(MsgField::name);
            quote! { #(#deserialized_payload),* }
        }
    }

    fn emit_payload_deserialization(&self) -> TokenStream {
        let sylvia = crate_module();

        if self
            .fields()
            .iter()
            .any(|field| field.contains_attribute(SylviaAttribute::Payload))
        {
            return quote! {};
        }

        let deserialized_names = self.fields().iter().skip(1).map(MsgField::name);
        quote! {
            let ( #(#deserialized_names),* ) = #sylvia ::cw_std::from_json(&payload)?;
        }
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
