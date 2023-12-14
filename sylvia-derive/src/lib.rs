use input::{ImplInput, TraitInput};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::fold::Fold;
use syn::{parse2, parse_quote, ItemImpl, ItemTrait, Path};

mod associated_types;
pub(crate) mod check_generics;
mod input;
mod interfaces;
mod message;
mod multitest;
mod parser;
mod querier;
mod remote;
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

/// Macro generating messages from contract trait.
///
/// ## Example usage
/// ```ignore
/// # use cosmwasm_std::Response;
///
/// # struct Ctx;
/// # struct Error;
///
/// # #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
/// # struct Member;
///
/// # #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
/// # struct AdminQueryResponse;
///
/// # #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
/// # struct MemberQueryResponse;
///
/// #[sylvia::interface(module=msg)]
/// trait Cw4 {
///     type Error: From<StdError>;
///
///     #[msg(exec)]
///     fn update_admin(&self, ctx: (DepsMut, Env, MessageInfo), admin: Option<String>) -> Result<Response, Self::Error>;
///
///     #[msg(exec)]
///     fn update_members(&self, ctx: (DepsMut, Env, MessageInfo), remove: Vec<String>, add: Vec<Member>)
///         -> Result<Response, Self::Error>;
///
///     #[msg(query)]
///     fn admin(&self, ctx: (Deps, Env)) -> Result<AdminQueryResponse, Error>;
///
///     #[msg(query)]
///     fn member(&self, ctx: (Deps, Env), addr: String, at_height: Option<u64>) -> Result<MemberQueryResponse, Error>;
/// }
/// ```
///
/// This would generate output like:
///
/// ```ignore
/// pub mod msg {
///     # #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
///     # struct Member;
///
///     #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
///     #[serde(rename_all = "snake_case")]
///     pub enum ExecMsg {
///         UpdateAdmin { admin: Option<String> },
///         UpdateMembers {
///             remove: Vec<String>,
///             add: Vec<Member>,
///         },
///         AddHook { addr: String },
///         RemoveHook { addr: String },
///     }
///
///     impl ExecMsg {
///         pub fn dispatch<C: Cw4>(contract: &C, ctx: (DepsMut, Env, MessageInfo))
///             -> Result<Response, C::Error>
///         {
///             // Some dispatching implementation
///         }
///     }
/// }
///
/// And similar `Query` structure for handling queries.
/// ```
///
/// ## Parameters
///
/// `interface` attribute takes optional parameters:
/// * `module` - defines module name, where all generated messages would be encapsulated; no
/// additional module would be created if not provided
///
/// ## Attributes
///
/// Messages structures are generated basing on interface trait method attributed with
/// `#[msg(msg_type, ...)`. Msg attribute takes as its first argument type of message it is
/// supposed to handle:
///   * `exec` - this is execute message variant
///   * `query` - this is query message variant
///
/// In case of query it is possible to pass second argument which is it's `ResponseType`.
/// This is required in case of aliased results wrapping their `ResponseType`.
/// Example for member query
///
/// ```ignore
///     #[msg(query, resp=MemberQueryResponse)]
///     fn member(&self, ctx: (Deps, Env), addr: String, at_height: Option<u64>) -> Result<MemberQueryResponse, Error>;
/// ```
///
/// For now `#[msg(...)]` attribute doesn't support anymore data on `#[interface]`
/// elements, but it may be extended in future.
#[cfg(not(tarpaulin_include))]
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

/// Macro generating messages from contract impl block.
///
/// ## Example usage
/// ```ignore
/// # use cosmwasm_std::Response;
///
/// # struct Ctx;
/// # struct Error;
///
/// # #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
/// # struct Cw4Group;
///
/// #[cw_derive::contract(module=msg)]
/// impl Cw4Group {
///     #[msg(instantiate, name="Instantiate")]
///     fn instantiate(&self, ctx: (DepsMut, Env, MessageInfo), admin: Option<String>)
///         -> Result<Response, Error>;
/// }
/// ```
///
/// This would generate output like:
///
/// ```ignore
/// pub mod msg {
///     # #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
///     # struct Cw4Group;
///
///     #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
///     #[serde(rename_all = "snake_case")]
///     pub struct Instantiate {
///         admin: Option<String>,
///     }
///
///     impl Instantiate {
///         fn dispatch(contract: &Cw4Group, ctx: (DepsMut, Env, MessageInfo), admin: Option<String>)
///             -> Result<Response, Error>
///         {
///             contract.instantiate(ctx, admin)
///         }
///     }
/// }
/// ```
///
/// ## Parameters
///
/// `contract` attribute takes optional parameters:
/// * `module` - defines module name, where all generated messages would be encapsulated; no
/// additional module would be created if not provided
///
/// ## Attributes
///
/// Messages structures are generated basing on specific implemented methods attributed with
/// `#[msg(msg_type, ...)`. Msg attribute takes as its first argument type of message it is
/// supposed to handle:
/// * `instantiate` - this is instantiation message handler. There should be always exactly one
/// * `exec` - this is execute message variant
/// * `query` - this is query message variant
/// * `migrate` - this is migrate message variant
/// handler for this kind of message.
/// In case of query it is possible to pass second argument which is it's `ResponseType`.
/// This is required in case of aliased results wrapping their `ResponseType`.
/// Example for member query
///
/// ```ignore
///     #[msg(query, resp=MemberQueryResponse)]
///     fn member(&self, ctx: (Deps, Env), addr: String, at_height: Option<u64>) -> Result<MemberQueryResponse, Error>
/// ```
#[cfg(not(tarpaulin_include))]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn contract(attr: TokenStream, item: TokenStream) -> TokenStream {
    contract_impl(attr.into(), item.into()).into()
}

fn contract_impl(attr: TokenStream2, item: TokenStream2) -> TokenStream2 {
    fn inner(attr: TokenStream2, item: TokenStream2) -> syn::Result<TokenStream2> {
        let attrs: parser::ContractArgs = parse2(attr)?;
        let input: ItemImpl = parse2(item)?;

        let expanded = ImplInput::new(&attrs, &input).process();
        let input = StripInput.fold_item_impl(input);

        Ok(quote! {
            #input

            #expanded
        })
    }

    inner(attr, item).unwrap_or_else(syn::Error::into_compile_error)
}

#[cfg(not(tarpaulin_include))]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn entry_points(attr: TokenStream, item: TokenStream) -> TokenStream {
    entry_points_impl(attr.into(), item.into()).into()
}

#[cfg(not(tarpaulin_include))]
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
