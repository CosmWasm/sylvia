//! This crate defines [Sylvia](https://docs.rs/sylvia/latest/sylvia) procedural macros.
//!
//! Please refer to the [Sylvia-book](https://cosmwasm.github.io/sylvia-book/index.html) on how to use these macros.

use input::{ImplInput, TraitInput};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::fold::Fold;
use syn::{parse2, parse_quote, ItemImpl, ItemTrait, Path};

mod associated_types;
pub(crate) mod check_generics;
mod executor;
mod input;
mod interfaces;
mod message;
mod message_type;
mod multitest;
mod parser;
mod querier;
mod strip_generics;
mod strip_input;
mod strip_self_path;
mod utils;
mod variant_descs;

use strip_input::StripInput;

use crate::message::EntryPoints;

#[cfg(not(test))]
pub(crate) fn crate_module() -> Path {
    use proc_macro_crate::{crate_name, FoundCrate};

    match crate_name("sylvia").expect("sylvia is not found in Cargo.toml") {
        FoundCrate::Itself => parse_quote!(sylvia),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::mixed_site());
            parse_quote!(#ident)
        }
    }
}

#[cfg(test)]
pub(crate) fn crate_module() -> Path {
    parse_quote!(sylvia)
}

/// Procedural macro generating messages from a contract trait.
/// Generates `sudo`, `exec` and `query` enum messages to be later used in contract implementation.
///
/// ## Example usage
///
/// ```rust
/// # use sylvia::cw_schema::cw_serde;
/// # use sylvia::cw_std::{Response, StdError};
/// # use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
/// #
/// # #[cw_serde]
/// # pub struct AdminQueryResponse;
/// #
/// ##[sylvia::interface]
/// pub trait SvInterface {
///    type Error: From<StdError>;
///
///    #[sv::msg(exec)]
///    fn update_admin(&self, ctx: ExecCtx, admin: Option<String>) -> Result<Response, Self::Error>;
///
///    #[sv::msg(query)]
///    fn admin(&self, ctx: QueryCtx) -> Result<AdminQueryResponse, Self::Error>;
///
///    #[sv::msg(sudo)]
///    fn remove_admin(&self, ctx: SudoCtx, #[serde(default)] admin: String) -> Result<Response, Self::Error>;
/// }
/// # fn main() {}
/// ```
///
/// This would generate output like:
///
/// ```rust
/// # use sylvia::cw_schema::cw_serde;
/// # use sylvia::cw_std::{Response, StdError, DepsMut, Env, MessageInfo};
/// # use sylvia::types::ExecCtx;
/// #
/// # pub trait SvInterface {
/// #    type Error: From<StdError>;
/// # }
/// #
/// pub mod sv {
/// #   use super::*;
/// #
///     #[derive(
///         sylvia::serde::Serialize,
///         sylvia::serde::Deserialize,
///         Clone,
///         Debug,
///         PartialEq,
///         sylvia::schemars::JsonSchema,
///     )]
///     #[serde(rename_all = "snake_case")]
///     pub enum ExecMsg {
///         UpdateAdmin { admin: Option<String> },
///     }
///
///     impl ExecMsg {
///         pub fn dispatch<C: SvInterface>(contract: &C, ctx: (DepsMut, Env, MessageInfo))
///             -> Result<Response, C::Error>
///         {
///             // Some dispatching implementation
/// #           Ok(Response::new())
///         }
///     }
/// }
/// # fn main() {}
/// ```
///
/// Similarly for `Query` and `Sudo` enum messages.
///
/// ## Associated types
///
/// Generics are not supported by the `interface` macro and won't be. Instead, you can define
/// associated types on an interface to allow users implementing it to customize the behavior to their liking.
///
/// Some names are however parsed and used in special contexts. Those are:
/// * `Error` - error type returned by interface methods. This one is required to be declared.
/// * `ExecC` - custom message type used in messages. Has to implement `cosmwasm_std::CustomMsg`.
/// * `QueryC` - custom query type used in messages. Has to implement `cosmwasm_std::CustomQuery`.
///
/// ```rust
/// # use sylvia::cw_schema::cw_serde;
/// # use sylvia::cw_std::{CustomMsg, CustomQuery, Response, StdError};
/// # use sylvia::types::ExecCtx;
/// #
/// ##[sylvia::interface]
/// pub trait SvInterface {
///    type Error: From<StdError>;
///    type ExecC: CustomMsg;
///    type QueryC: CustomQuery;
///
///    #[sv::msg(exec)]
///    fn update_admin(&self, ctx: ExecCtx<Self::QueryC>, admin: Option<String>) -> Result<Response<Self::ExecC>, Self::Error>;
/// }
/// # fn main() {}
/// ```
///
/// Although it's not required to define `ExecC` and `QueryC` types, it's recommended to do so to allow
/// the users of the interface to customize the behavior to their liking.
///
/// If however you want to restrict the interface to use specific custom message and query types,
/// you can do so using `#[sv::custom(msg=..., query=...)]` attribute explained below.
///
/// ## Attributes
///
/// `Interface` macro supports multiple attributes to customize the behavior of generated messages.
///
/// ### `sv::msg(...)`
///
/// Messages structures are generated basing on interface trait method attributed with
/// `#[sv::msg(msg_type)]`. Msg attribute takes as its first argument type of message it is
/// supposed to handle:
///   * `exec` - execute message variant
///   * `query` - query message variant
///   * `sudo` - sudo message variant
///
/// In the case of a query, it is possible to pass a second argument which is its `ResponseType`.
/// This is required in case of aliased results wrapping their `ResponseType` to properly
/// implement [QueryResponses](https://docs.rs/cosmwasm-schema/latest/cosmwasm_schema/trait.QueryResponses.html).
///
/// ```rust
/// # use sylvia::cw_schema::cw_serde;
/// # use sylvia::cw_std::{Response, StdError};
/// # use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
/// #
/// # #[cw_serde]
/// # pub struct AdminQueryResponse;
/// #
/// pub type AdminResult<ErrorT> = Result<AdminQueryResponse, ErrorT>;
///
/// ##[sylvia::interface]
/// pub trait SvInterface {
///    type Error: From<StdError>;
///
///    #[sv::msg(exec)]
///    fn update_admin(&self, ctx: ExecCtx, admin: Option<String>) -> Result<Response, Self::Error>;
///
///    #[sv::msg(query, resp=AdminQueryResponse)]
///    fn admin(&self, ctx: QueryCtx) -> AdminResult<Self::Error>;
///
///    #[sv::msg(sudo)]
///    fn remove_admin(&self, ctx: SudoCtx, admin: String) -> Result<Response, Self::Error>;
/// }
/// # fn main() {}
/// ```
///
/// ### `sv::custom(msg=..., query=...)`
///
/// Allows restricting interface to use specific
/// custom message and query types. If used with `ExecC` and `QueryC` associated
/// types `sv::custom(...)` attribute has priority in defining custom types.
///
/// ```rust
/// # use sylvia::cw_schema::cw_serde;
/// # use sylvia::cw_std::{CustomMsg, CustomQuery, Response, StdError};
/// # use sylvia::types::{ExecCtx, QueryCtx, SudoCtx};
/// #
/// ##[cw_serde]
/// pub enum SvCustomMsg {}
///
/// ##[cw_serde]
/// pub enum SvCustomQuery {}
///
/// impl CustomMsg for SvCustomMsg {}
/// impl CustomQuery for SvCustomQuery {}
///
/// ##[sylvia::interface]
/// ##[sv::custom(msg=SvCustomMsg, query=SvCustomQuery)]
/// pub trait SvInterface {
///    type Error: From<StdError>;
///    type ExecC: CustomMsg;
///    type QueryC: CustomQuery;
///
///    #[sv::msg(exec)]
///    fn update_admin(&self, ctx: ExecCtx<SvCustomQuery>, admin: Option<String>) -> Result<Response<SvCustomMsg>, Self::Error>;
/// }
/// # fn main() {}
/// ```
///
/// ### `sv::msg_attr(msg_type, {...})`
///
/// This attribute can be used for the whole `trait Interface {}` block and
/// for the following message types: `exec`, `query` and `sudo`.
/// The `{...}` part will be forwarded as an attribute `#[{...}]` to the
/// given message type (enum or struct).
///
/// ### `sv::attr({...})`
///
/// Forwards variant's attribute to the specific enum's field in the
/// generated message type. It can be used along with `sv::msg(...)`
/// and only for message types variants that resolves in an enum field,
/// i.e. `exec`, `query` and `sudo`.
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    interface_impl(attr.into(), item.into()).into()
}

fn interface_impl(_attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    fn inner(item: TokenStream2) -> syn::Result<TokenStream2> {
        let input: ItemTrait = parse2(item)?;

        let expanded = TraitInput::new(&input).process();
        let input = StripInput.fold_item_trait(input);

        Ok(quote! {
            #input

            #expanded
        })
    }

    inner(item).unwrap_or_else(syn::Error::into_compile_error)
}

/// Procedural macro generating messages from contract impl block.
/// Generates `instantiate`, `migrate`, `reply`, `sudo`, `exec` and `query`
/// enum messages to be later used in contract implementation.
///
/// ## Example usage
/// ```rust
/// # use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx};
/// # use sylvia::cw_std::{Response, StdError, Reply};
/// # use cw_storage_plus::Item;
/// # use thiserror::Error;
/// #
/// # #[derive(Error, Debug, PartialEq)]
/// # pub enum ContractError {
/// #     #[error("{0}")]
/// #     Std(#[from] StdError),
/// # }
/// # pub struct ContractData;
/// #
/// #
/// pub struct SvContract {
///     data: Item<ContractData>,
/// }
///
/// ##[sylvia::contract]
/// ##[sv::error(ContractError)]
/// impl SvContract {
///     pub const fn new() -> Self {
///         Self {
///             data: Item::new("data"),
///         }
///     }
///
///     #[sv::msg(instantiate)]
///     fn instantiate(&self, ctx: InstantiateCtx, admin: Option<String>) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
///     }
///
///     #[sv::msg(exec)]
///     fn execute(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
///     }
///
///     #[sv::msg(query)]
///     fn query(&self, ctx: QueryCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
///     }
///
///     #[sv::msg(migrate)]
///     fn migrate(&self, ctx: MigrateCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
///     }
///
///     #[sv::msg(reply)]
///     fn reply(&self, ctx: ReplyCtx, reply: Reply) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
///     }
///
///     #[sv::msg(sudo)]
///     fn sudo(&self, ctx: SudoCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
///     }
/// }
/// # fn main() {}
/// ```
///
/// This would generate output like:
///
/// ```rust
/// # use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx};
/// # use sylvia::cw_std::{Response, StdError};
/// # use cw_storage_plus::Item;
/// # use thiserror::Error;
/// #
/// # #[derive(Error, Debug, PartialEq)]
/// # pub enum ContractError {
/// #     #[error("{0}")]
/// #     Std(#[from] StdError),
/// # }
/// # pub struct ContractData;
/// #
/// # pub struct SvContract {
/// #    data: Item<ContractData>,
/// # }
/// #
/// # impl SvContract {
/// #     pub const fn new() -> Self {
/// #        Self {
/// #            data: Item::new("data"),
/// #        }
/// #    }
/// #
/// #    fn instantiate(&self, ctx: InstantiateCtx, admin: Option<String>) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    fn execute(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    fn query(&self, ctx: QueryCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    fn migrate(&self, ctx: MigrateCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    fn reply(&self, ctx: ReplyCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    fn sudo(&self, ctx: SudoCtx) -> Result<Response, ContractError> {
/// #        Ok(Response::new())
/// #    }
/// # }
/// #
/// pub mod sv {
/// #   use super::*;
/// #
///     #[allow(clippy::derive_partial_eq_without_eq)]
///     #[derive(
///         sylvia::serde::Serialize,
///         sylvia::serde::Deserialize,
///         Clone,
///         Debug,
///         PartialEq,
///         sylvia::schemars::JsonSchema,
///     )]
///     #[serde(rename_all = "snake_case")]
///     pub struct InstantiateMsg {
///         pub admin: Option<String>,
///     }
///     impl InstantiateMsg {
///         pub fn new(admin: Option<String>) -> Self {
///             Self { admin }
///         }
///         pub fn dispatch(
///             self,
///             contract: &SvContract,
///             ctx: (
///                 sylvia::cw_std::DepsMut<sylvia::cw_std::Empty>,
///                 sylvia::cw_std::Env,
///                 sylvia::cw_std::MessageInfo,
///             ),
///         ) -> Result<Response, ContractError> {
///             let Self { admin } = self;
///             contract
///                 .instantiate(Into::into(ctx), admin)
///                 .map_err(Into::into)
///         }
///     }
/// }
/// # fn main() {}
/// ```
///
/// And appropriate messages for `exec`, `query`, `migrate`, `reply` and `sudo` variants.
///
/// ## Attributes
///
/// `Contract` macro supports multiple attributes to customize the behavior of generated messages.
///
/// ### `sv::msg(...)`
///
/// Message structures are generated based on specific implemented methods attributed with
/// `#[sv::msg(msg_type)]`. Msg attribute takes as its first argument type of message it is
/// supposed to handle:
/// * `instantiate` - instantiation message handler. There should be always exactly one
/// * `exec` - execute message variant
/// * `query` - query message variant
/// * `migrate` - migrate message variant
/// * `reply` - reply message variant
/// * `sudo` - sudo message variant
///
/// In the case of a query, it is possible to pass a second argument which is its `ResponseType`.
/// This is required in case of aliased results wrapping their `ResponseType` to properly
/// implement `QueryResponses`.
///
/// ```rust
/// # use sylvia::types::{InstantiateCtx, QueryCtx};
/// # use sylvia::cw_std::{Response, StdError};
/// # use sylvia::cw_schema::cw_serde;
/// # use cw_storage_plus::Item;
/// # use thiserror::Error;
/// #
/// # #[derive(Error, Debug, PartialEq)]
/// # pub enum ContractError {
/// #     #[error("{0}")]
/// #     Std(#[from] StdError),
/// # }
/// # pub struct ContractData;
/// #
/// # #[cw_serde]
/// # pub struct QueryResponse;
/// #
/// pub struct SvContract {
///     data: Item<ContractData>,
/// }
///
/// ##[sylvia::contract]
/// ##[sv::error(ContractError)]
/// impl SvContract {
///     pub const fn new() -> Self {
///         Self {
///             data: Item::new("data"),
///         }
///     }
///
///     #[sv::msg(instantiate)]
///     fn instantiate(&self, ctx: InstantiateCtx, admin: Option<String>) -> Result<Response, ContractError> {
///         Ok(Response::new())
///     }
///
///     #[sv::msg(query, resp=QueryResponse)]
///     fn query(&self, ctx: QueryCtx) -> Result<QueryResponse, ContractError> {
///         // Some query implementation
/// #       Ok(QueryResponse)
///     }
/// }
/// # fn main() {}
/// ```
///
/// ### `sv::custom(msg=..., query=...)`
///
/// Allows restricting interface to use specific
/// custom message and query types. If used with `ExecC` and `QueryC` associated
/// types `sv::custom(...)` attribute has priority in defining custom types.
///
/// ### `sv::error(error_type)`
///
/// Allows specifing custom error type for the contract. Default is `cosmwasm_std::StdError`.
///
/// ### `sv::override_entry_point(entry_point_type=<path_to_entry_point(msg_path)>`
///
/// Allows overriding default entry point for specific message type.
/// Used in case there is a need to provide some custom functionality inside the entry point.
/// Specified entry point will be used in generated `multitest` helpers
///
/// ```rust
/// # use sylvia::types::InstantiateCtx;
/// # use sylvia::cw_std::{Response, StdResult, MessageInfo, DepsMut, Env, entry_point};
/// # use sylvia::cw_schema::cw_serde;
/// #
/// pub struct SvContract;
///
/// #[cw_serde]
/// pub enum CustomExecMsg {}
///
/// ##[entry_point]
/// pub fn execute(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: CustomExecMsg,
/// ) -> StdResult<Response> {
/// #   Ok(Response::new())
/// }
///
///
/// ##[sylvia::contract]
/// ##[sv::override_entry_point(exec=execute(CustomExecMsg))]
/// impl SvContract {
/// #    pub const fn new() -> Self {
/// #        Self
/// #    }
/// #
/// #    #[sv::msg(instantiate)]
/// #    fn instantiate(&self, ctx: InstantiateCtx, admin: Option<String>) -> StdResult<Response> {
/// #        Ok(Response::new())
/// #    }
/// }
/// # fn main() {}
/// ```
/// ### `sv::messages(path_to_interface)`
///
/// Used to declare interfaces implemented on the contract.
/// Required for the contract to be able to handle an interface message.
///
/// ```rust
/// # use sylvia::types::InstantiateCtx;
/// # use sylvia::cw_std::{Response, StdError, StdResult};
/// # use cw_storage_plus::Item;
/// #
/// pub mod sv_interface {
/// #   use sylvia::cw_std::StdError;
/// #
///     ##[sylvia::interface]
///     pub trait SvInterface {
///         type Error: From<StdError>;
///     }
/// }
///
/// impl sv_interface::SvInterface for SvContract {
///     type Error = StdError;
/// }
///
/// pub struct SvContract;
///
/// ##[sylvia::contract]
/// ##[sv::messages(sv_interface)]
/// impl SvContract {
/// #     pub const fn new() -> Self {
/// #         Self
/// #     }
/// #
/// #     #[sv::msg(instantiate)]
/// #     fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
/// #         Ok(Response::new())
/// #     }
/// }
/// # fn main() {}
/// ```
///
/// In case an interface has different name than a module in which its defined
/// you have to pass the name of an interface as below:
///
/// ```rust
/// # use sylvia::types::InstantiateCtx;
/// # use sylvia::cw_std::{Response, StdError, StdResult};
/// # use cw_storage_plus::Item;
/// #
/// pub mod interface {
/// #   use sylvia::cw_std::StdError;
/// #
///     ##[sylvia::interface]
///     pub trait SvInterface {
///         type Error: From<StdError>;
///     }
/// }
///
/// impl interface::SvInterface for SvContract {
///     type Error = StdError;
/// }
///
/// pub struct SvContract;
///
/// ##[sylvia::contract]
/// ##[sv::messages(interface as SvInterface)]
/// impl SvContract {
/// #     pub const fn new() -> Self {
/// #         Self
/// #     }
/// #
/// #     #[sv::msg(instantiate)]
/// #     fn instantiate(&self, ctx: InstantiateCtx) -> StdResult<Response> {
/// #         Ok(Response::new())
/// #     }
/// }
/// # fn main() {}
/// ```
///
/// ### `sv::msg_attr(msg_type, {...})`
///
/// This attribute can be used for the whole `impl Contract {}` block and
/// for every message type: `exec`, `query`, `sudo`, `instantiate`,
/// `migrate` and `reply`. The `{...}` part will be forwarded as an
/// attribute `#[{...}]` to the given message type (enum or struct).
///
/// ### `sv::attr({...})`
///
/// Forwards variant's attribute to the specific enum's field in the
/// generated message type. It can be used along with `sv::msg(...)`
/// and only for message types variants that resolves in an enum field,
/// i.e. `exec`, `query` and `sudo`.
///
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract_impl(attr.into(), item.into()).into()
}

fn contract_impl(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    fn inner(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
        let input: ItemImpl = parse2(item)?;
        let expanded = if attr.is_empty() {
            ImplInput::new(&input).process()
        } else {
            quote! {}
        };
        let input = StripInput.fold_item_impl(input);

        Ok(quote! {
            #input

            #expanded
        })
    }

    inner(attr, item).unwrap_or_else(syn::Error::into_compile_error)
}

/// Procedural macro generating cosmwasm entry points from contract impl block.
/// By default generates `execute`, `instantiate`, `sudo`, `query` entry points.
///
/// ## Example usage
/// ```rust
/// # use sylvia::types::{ExecCtx, InstantiateCtx, MigrateCtx, QueryCtx, ReplyCtx, SudoCtx};
/// # use sylvia::cw_std::{Reply, Response, StdResult};
/// #
/// pub struct SvContract;
///
/// ##[sylvia::entry_points]
/// ##[sylvia::contract]
/// impl SvContract {
///     pub const fn new() -> Self {
///         Self
///     }
///
///     #[sv::msg(instantiate)]
///     fn instantiate(&self, ctx: InstantiateCtx, admin: Option<String>) -> StdResult<Response> {
/// #        Ok(Response::new())
///     }
/// #
/// #    #[sv::msg(exec)]
/// #    fn execute(&self, ctx: ExecCtx) -> StdResult<Response> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    #[sv::msg(query)]
/// #    fn query(&self, ctx: QueryCtx) -> StdResult<Response> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    #[sv::msg(migrate)]
/// #    fn migrate(&self, ctx: MigrateCtx) -> StdResult<Response> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    #[sv::msg(reply)]
/// #    fn reply(&self, ctx: ReplyCtx, reply: Reply) -> StdResult<Response> {
/// #        Ok(Response::new())
/// #    }
/// #
/// #    #[sv::msg(sudo)]
/// #    fn sudo(&self, ctx: SudoCtx) -> StdResult<Response> {
/// #        Ok(Response::new())
/// #    }
/// }
/// # fn main() {}
/// ```
///
/// ## Generics
///
/// `Cosmwasm` entry point has to be implemented with concrete types.
/// In case your contract uses some generic types you have to specify concrete types
/// used in their place in the `entry_points` macro attribute `generics`.
///
/// ```rust
/// # use sylvia::types::InstantiateCtx;
/// # use sylvia::cw_std::{Response, StdResult};
/// # use sylvia::cw_schema::cw_serde;
/// # use cw_storage_plus::Item;
/// # use std::marker::PhantomData;
/// #
/// # #[cw_serde]
/// # pub struct SvCustomMsg;
/// # impl sylvia::cw_std::CustomMsg for SvCustomMsg {}
/// #
/// pub struct SvContract<InstantiateT, DataT> {
///     data: Item<DataT>,
///     _phantom: PhantomData<InstantiateT>,
/// }
///
/// ##[sylvia::entry_points(generics<SvCustomMsg, SvCustomMsg>)]
/// ##[sylvia::contract]
/// impl<InstantiateT, DataT> SvContract<InstantiateT, DataT>
///     where InstantiateT: sylvia::types::CustomMsg + 'static,
///         DataT: 'static
/// {
///     pub const fn new() -> Self {
///         Self {
///             data: Item::new("data"),
///             _phantom: PhantomData,
///         }
///     }
///
///     #[sv::msg(instantiate)]
///     fn instantiate(&self, ctx: InstantiateCtx, instantiate_data: InstantiateT) -> StdResult<Response> {
/// #        Ok(Response::new())
///     }
/// }
/// # fn main() {}
/// ```
///
#[proc_macro_error]
#[proc_macro_attribute]
pub fn entry_points(attr: TokenStream, item: TokenStream) -> TokenStream {
    entry_points_impl(attr.into(), item.into()).into()
}

fn entry_points_impl(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    fn inner(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
        let attrs: parser::EntryPointArgs = parse2(attr)?;
        let input: ItemImpl = parse2(item)?;
        let expanded = EntryPoints::new(&input, attrs).emit();

        Ok(quote! {
            #input

            #expanded
        })
    }

    inner(attr, item).unwrap_or_else(syn::Error::into_compile_error)
}

#[cfg(test)]
mod test {
    use std::{env, fs};

    use sylvia_runtime_macros::emulate_attribute_expansion_fallible;

    use crate::{contract_impl, interface_impl};

    // Test expanding macros in sylvia crate tests, to calculate generating code coverage
    #[test]
    fn sylvia_test_cov() {
        let mut path = env::current_dir().unwrap();
        path.push("..");
        path.push("sylvia");
        path.push("tests");

        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();

            if entry.file_type().unwrap().is_file() {
                let file = fs::File::open(entry.path()).unwrap();
                emulate_attribute_expansion_fallible(file, "interface", interface_impl).unwrap();

                let file = fs::File::open(entry.path()).unwrap();
                emulate_attribute_expansion_fallible(file, "contract", contract_impl).unwrap();
            }
        }
    }
}
