use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::fold::Fold;
use syn::{parse_quote, GenericParam, Ident, ItemImpl, Type};

use crate::crate_module;
use crate::fold::StripGenerics;
use crate::parser::attributes::msg::ReplyOn;
use crate::parser::MsgType;
use crate::types::msg_variant::{MsgVariant, MsgVariants};

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

        quote! {
            #(#unique_handlers)*

            #dispatch
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
            .for_each(|(reply_id, function_name, reply_on)| {
                match reply_data
                    .iter_mut()
                    .find(|existing_data| existing_data.reply_id == reply_id)
                {
                    Some(existing_data)
                        if existing_data
                            .handlers
                            .iter()
                            .any(|(_, prev_reply_on)| prev_reply_on.excludes(&reply_on)) =>
                    {
                        existing_data.handlers.iter().for_each(
                            |(prev_function_name, prev_reply_on)| {
                                emit_error!(reply_id.span(), "Duplicated reply handler.";
                                    note = existing_data.reply_id.span() => format!("Previous definition of handler={} for reply_on={} defined on `fn {}()`", existing_data.reply_id.to_string(), prev_reply_on, prev_function_name);
                                )
                            },
                        )
                    }
                    Some(existing_data) => existing_data.handlers.push((function_name, reply_on)),
                    None => reply_data.push(ReplyData::new(reply_id, function_name, reply_on)),
                }
            });

        reply_data
    }
}

/// Maps single reply id with its handlers.
struct ReplyData<'a> {
    pub reply_id: Ident,
    pub handlers: Vec<(&'a Ident, ReplyOn)>,
}

impl<'a> ReplyData<'a> {
    pub fn new(reply_id: Ident, function_name: &'a Ident, reply_on: ReplyOn) -> Self {
        Self {
            reply_id,
            handlers: vec![(function_name, reply_on)],
        }
    }

    /// Emits success and failure match arms for a single `ReplyId`.
    fn emit_match_arms(&self, contract: &Type, generics: &[&GenericParam]) -> TokenStream {
        let Self { reply_id, handlers } = self;

        let contract_turbo: Type = if !generics.is_empty() {
            let contract_name = StripGenerics.fold_type((contract.clone()).clone());
            parse_quote! { #contract_name :: < #(#generics),* > }
        } else {
            parse_quote! { #contract }
        };

        let success_match_arm = emit_success_match_arm(handlers, &contract_turbo);
        let failure_match_arm = emit_failure_match_arm(handlers, &contract_turbo);

        quote! {
            #reply_id => {
                match result {
                    #success_match_arm
                    #failure_match_arm
                }
            }
        }
    }
}

/// Emits match arm for [ReplyOn::Success].
/// In case neither [ReplyOn::Success] nor [ReplyOn::Always] is present, `Response::events`
/// and `Response::data` are forwarded in the `Response`
fn emit_success_match_arm(handlers: &[(&Ident, ReplyOn)], contract_turbo: &Type) -> TokenStream {
    let sylvia = crate_module();

    match handlers
        .iter()
        .find(|(_, reply_on)| reply_on == &ReplyOn::Success || reply_on == &ReplyOn::Always)
    {
        Some((function_name, reply_on)) if reply_on == &ReplyOn::Success => quote! {
            #sylvia ::cw_std::SubMsgResult::Ok(sub_msg_resp) => {
                #[allow(deprecated)]
                let #sylvia ::cw_std::SubMsgResponse { events, data, msg_responses} = sub_msg_resp;
                #contract_turbo ::new(). #function_name ((deps, env, gas_used, events, msg_responses).into(), data, payload)
            }
        },
        Some((function_name, reply_on)) if reply_on == &ReplyOn::Always => quote! {
            #sylvia ::cw_std::SubMsgResult::Ok(_) => {
                #contract_turbo ::new(). #function_name ((deps, env, gas_used, vec![], vec![]).into(), result, payload)
            }
        },
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
fn emit_failure_match_arm(handlers: &[(&Ident, ReplyOn)], contract_turbo: &Type) -> TokenStream {
    let sylvia = crate_module();

    match handlers
        .iter()
        .find(|(_, reply_on)| reply_on == &ReplyOn::Failure || reply_on == &ReplyOn::Always)
    {
        Some((function_name, reply_on)) if reply_on == &ReplyOn::Failure => quote! {
            #sylvia ::cw_std::SubMsgResult::Err(error) => {
                #contract_turbo ::new(). #function_name ((deps, env, gas_used, vec![], vec![]).into(), error, payload)
            }
        },
        Some((function_name, reply_on)) if reply_on == &ReplyOn::Always => quote! {
            #sylvia ::cw_std::SubMsgResult::Err(_) => {
                #contract_turbo ::new(). #function_name ((deps, env, gas_used, vec![], vec![]).into(), result, payload)
            }
        },
        _ => quote! {
            #sylvia ::cw_std::SubMsgResult::Err(error) => {
                Err(sylvia::cw_std::StdError::generic_err(error)).map_err(Into::into)
            }
        },
    }
}

trait ReplyVariant<'a> {
    fn as_handlers(&'a self) -> Vec<&'a Ident>;
    fn as_reply_data(&self) -> Vec<(Ident, &Ident, ReplyOn)>;
}

impl<'a> ReplyVariant<'a> for MsgVariant<'a> {
    fn as_handlers(&'a self) -> Vec<&'a Ident> {
        if self.msg_attr().handlers().is_empty() {
            return vec![self.function_name()];
        }
        self.msg_attr().handlers().iter().collect()
    }

    fn as_reply_data(&self) -> Vec<(Ident, &Ident, ReplyOn)> {
        self.as_handlers()
            .iter()
            .map(|handler| {
                (
                    handler.as_reply_id(),
                    self.function_name(),
                    self.msg_attr().reply_on(),
                )
            })
            .collect()
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
