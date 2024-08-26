use proc_macro2::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::fold::Fold;
use syn::{parse_quote, GenericParam, Ident, Type};

use crate::crate_module;
use crate::fold::StripSelfPath;
use crate::parser::attributes::msg::MsgType;
use crate::parser::Customs;

impl MsgType {
    pub fn emit_ctx_type(self, query_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                (#sylvia ::cw_std::DepsMut< #query_type >, #sylvia ::cw_std::Env, #sylvia ::cw_std::MessageInfo)
            },
            Migrate | Reply | Sudo => quote! {
                (#sylvia ::cw_std::DepsMut< #query_type >, #sylvia ::cw_std::Env)
            },
            Query => quote! {
                (#sylvia ::cw_std::Deps< #query_type >, #sylvia ::cw_std::Env)
            },
        }
    }

    pub fn emit_ctx_dispatch_values(self, customs: &Customs) -> TokenStream {
        use MsgType::*;

        match (self, customs.has_query) {
            (Exec, true) => quote! {
                (ctx.0.into_empty(), ctx.1, ctx.2)
            },
            (Query, true) | (Sudo, true) => quote! {
                (ctx.0.into_empty(), ctx.1)
            },
            _ => quote! { ctx },
        }
    }

    pub fn emit_ctx_params(self, query_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate => quote! {
                deps: #sylvia ::cw_std::DepsMut< #query_type>, env: #sylvia ::cw_std::Env, info: #sylvia ::cw_std::MessageInfo
            },
            Migrate | Reply | Sudo => quote! {
                deps: #sylvia ::cw_std::DepsMut< #query_type>, env: #sylvia ::cw_std::Env
            },
            Query => quote! {
                deps: #sylvia ::cw_std::Deps< #query_type>, env: #sylvia ::cw_std::Env
            },
        }
    }

    pub fn emit_ep_name(self) -> Ident {
        match self {
            Self::Exec => parse_quote! { execute },
            Self::Instantiate => parse_quote! { instantiate },
            Self::Migrate => parse_quote! { migrate },
            Self::Sudo => parse_quote! { sudo },
            Self::Reply => parse_quote! { reply },
            Self::Query => parse_quote! { query },
        }
    }

    pub fn emit_ctx_values(self) -> TokenStream {
        use MsgType::*;

        match self {
            Exec | Instantiate => quote! { deps, env, info },
            Migrate | Reply | Query | Sudo => quote! { deps, env },
        }
    }

    /// Emits type which should be returned by dispatch function for this kind of message
    pub fn emit_result_type(self, msg_type: &Type, err_type: &Type) -> TokenStream {
        use MsgType::*;

        let sylvia = crate_module();

        match self {
            Exec | Instantiate | Migrate | Reply | Sudo => {
                quote! {
                    std::result::Result< #sylvia:: cw_std::Response <#msg_type>, #err_type>
                }
            }
            Query => quote! {
                std::result::Result<#sylvia ::cw_std::Binary, #err_type>
            },
        }
    }

    pub fn emit_msg_wrapper_name(&self) -> Ident {
        match self {
            MsgType::Exec => parse_quote! { ContractExecMsg },
            MsgType::Query => parse_quote! { ContractQueryMsg },
            MsgType::Sudo => parse_quote! { ContractSudoMsg },
            _ => self.emit_msg_name(),
        }
    }

    pub fn emit_msg_name(&self) -> Ident {
        match self {
            MsgType::Exec => parse_quote! { ExecMsg },
            MsgType::Query => parse_quote! { QueryMsg },
            MsgType::Instantiate => parse_quote! { InstantiateMsg },
            MsgType::Migrate => parse_quote! { MigrateMsg },
            MsgType::Reply => parse_quote! { ReplyMsg },
            MsgType::Sudo => parse_quote! { SudoMsg },
        }
    }

    pub fn as_accessor_wrapper_name(&self) -> Type {
        match self {
            MsgType::Exec => parse_quote! { ContractExec },
            MsgType::Query => parse_quote! { ContractQuery },
            MsgType::Sudo => parse_quote! { ContractSudo },
            _ => self.as_accessor_name(),
        }
    }

    pub fn as_accessor_name(&self) -> Type {
        match self {
            MsgType::Instantiate => parse_quote! { Instantiate },
            MsgType::Exec => parse_quote! { Exec },
            MsgType::Query => parse_quote! { Query },
            MsgType::Migrate => parse_quote! { Migrate },
            MsgType::Sudo => parse_quote! { Sudo },
            MsgType::Reply => parse_quote! { Reply },
        }
    }

    pub fn emit_phantom_variant(&self, generics: &[&GenericParam]) -> TokenStream {
        match self {
            _ if generics.is_empty() => quote! {},
            MsgType::Query => quote! {
                #[serde(skip)]
                #[returns(( #(#generics,)* ))]
                _Phantom(std::marker::PhantomData<( #(#generics,)* )>),
            },
            _ => quote! {
                #[serde(skip)]
                _Phantom(std::marker::PhantomData<( #(#generics,)* )>),
            },
        }
    }

    pub fn emit_derive_call(&self) -> TokenStream {
        let sylvia = crate_module();
        let cw_schema = quote! { #sylvia:: cw_schema }.to_string();
        let schemars = quote! { #sylvia:: cw_schema::schemars }.to_string();
        let serde = quote! { #sylvia:: serde }.to_string();

        match self {
            MsgType::Query => quote! {
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema, #sylvia:: cw_schema::QueryResponses)]
                #[schemars(crate = #schemars )]
                #[query_responses(crate = #cw_schema )]
                #[serde(crate = #serde )]
            },
            _ => quote! {
                #[derive(#sylvia ::serde::Serialize, #sylvia ::serde::Deserialize, Clone, Debug, PartialEq, #sylvia ::schemars::JsonSchema)]
                #[schemars(crate = #schemars )]
                #[serde(crate = #serde )]
            },
        }
    }

    pub fn emit_dispatch_leg(&self, function_name: &Ident, args: &Vec<Ident>) -> TokenStream {
        use MsgType::*;
        let sylvia = crate_module();

        match self {
            Exec | Sudo => quote! {
                contract.#function_name(Into::into(ctx), #(#args),*).map_err(Into::into)
            },
            Query => quote! {
                #sylvia ::cw_std::to_json_binary(&contract.#function_name(Into::into(ctx), #(#args),*)?).map_err(Into::into)
            },
            Instantiate | Migrate | Reply => {
                emit_error!(function_name.span(), "Internal Error";
                note = "Dispatch leg should be called only for `Enum` type messages.");
                quote! {}
            }
        }
    }

    pub fn emit_returns_attribute(&self, return_type: &Option<Type>) -> TokenStream {
        match (self, return_type) {
            (MsgType::Query, Some(return_type)) => {
                let stripped_return_type = StripSelfPath.fold_type(return_type.clone());
                quote! { #[returns(#stripped_return_type)] }
            }
            _ => quote! {},
        }
    }
}
