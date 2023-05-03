# Sylvia Framework

Sylvia is the old name meaning Spirit of The Wood.

Sylvia is the Roman goddess of the forest.

Sylvia is also a framework created to give you the abstraction-focused and
scalable solution for building your CosmWasm Smart Contracts. Find your way
into the forest of Cosmos ecosystem. We provide you with the toolset, so instead
of focusing on the raw structure of your contract, you can create them in proper
and idiomatic Rust and then just let cargo make sure that they are sound.

Learn more about sylvia in [the book](https://cosmwasm.github.io/sylvia-book/index.html)

## The approach

[CosmWasm](https://cosmwasm.com/) ecosystem core provides the base building
blocks for smart contracts - the
[cosmwasm-std](https://crates.io/crates/cosmwasm-std) for basic CW bindings, the
[cw-storage-plus](https://crates.io/crates/cw-storage-plus) for easier state management,
and the [cw-multi-test](https://crates.io/crates/cw-multi-test) for testing them.
Sylvia framework is built on top of them, so for creating contracts, you don't
have to think about message structure, how their API is (de)serialized, or how
to handle message dispatching. Instead, the API of your contract is a set of
traits you implement on your SC type. The framework generates things like entry
point structures, functions dispatching the messages, or even helpers for multitest.
It allows for better control of interfaces, including validating their completeness
in compile time.

Also, as a side effect, as Sylvia has all the knowledge about the contract API structure,
it can generate many helpers - utilities for multitests or even queriers.

## Using in contracts

Fist you need your contract crate, which should be a library crate:

```shell
$ cargo new --lib ./my-crate
     Created library `./my-crate` package

```

To use sylvia in the contract, you need to add couple dependencies - sylvia itself,
and additionally: `cosmwasm-schema`, `schemars` and `cosmwasm_std`.

```shell
$ cargo add syvia cosmwasm-schema schemars cosmwasm-std
...
```

You should also make sure your crate is compiling as `cdylib`, setting the proper
crate type in `Cargo.toml`. I also like to add `rlib` there, so it is possible to
use the contract as the dependency. Example `Cargo.toml`:

```toml
[package]
name = "my-crate"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
cosmwasm-schema = "1.2.5"
cosmwasm-std = "1.2.5"
schemars = "0.8.12"
serde = "1.0.160"
sylvia = "0.3.2"
```

To build your contract as wasm you can use:

```rust
$ targo build --target wasm32-unknown-unknown
...
```

## Contract type

In Sylvia, we define our contracts as structures:

```rust
pub struct MyContract;
```

The next step is to create an instantiation message for the contract we have:

```rust
use sylvia::contract;
use sylvia::types::ExecCtx;
use sylvia::cw_std::{StdResult, Response};

#[contract]
impl MyContract {
    #[msg(instantiate)]
    pub fn instantiate(&self, _ctx: ExecCtx) -> StdResult<Response> {
        Ok(Response::new())
    }
}
```

This immediately creates the InstantiateMessage type in the same module you created
a contract struct. It looks like this:

```rust
struct InstantiateMsg {}
```

There are no fields there at this point, but they will be when we need them.

For now, we need this message to create a contract instantiate entry point for CosmWasm:

```rust
use sylvia::cw_std::{entry_point, DepsMut, Env, MessageInfo};

#[entry_point]
fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    msg.dispatch(&MyContract, (deps, env, info))
}
```

Now we would like to do something useful in the contract instantiation. Let's
start using the [cw-storage-plus](https://docs.rs/cw-storage-plus/1.0.1/cw_storage_plus/)
to add state to the contract (remember to add it as dependency):

```rust
use cw_storage_plus::Item;

struct MyContract<'a> {
    pub counter: Item<'a, u64>,
}

#[contract]
impl MyContract<'_> {
    pub const fn new() -> MyContract<'static> {
        MyContract {
            counter: Item::new("counter")
        }
    }

    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: ExecCtx) -> StdResult<Response> {
        self.counter.save(ctx.deps.storage, &0)?;

        Ok(Response::new())
    }
}

const CONTRACT: MyContract = MyContract::new();

#[entry_point]
fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> StdResult<Response> {
    msg.dispatch(&CONTRACT, (deps, env, info))
}
```

We need to add this generic lifetime because of an optimization in storage plus -
it doesn't want to take an owned string, as we often pass there a static string,
but it also doesn't want to fix the `'static` ownership. 99% of the time, you can
get away with passing just `'static` as the first `Item` generic argument, but I
find it more convenient to introduce this "proxy" lifetime passing everywhere. I
eliminate it in the `new` constructor, where I create the storage-plus accessors
giving them proper keys.

Note that I make the contract constructor const - it is possible as all my storage
accessors are const, allowing me to create a single global contract to access entry
points. Still, creating a contract on the entry point itself is possible - it is
just a matter of taste.

Now let's pass the initial counter state as a function argument:

```rust
#[contract]
impl MyContract<'_> {
    #[msg(instantiate)]
    pub fn instantiate(&self, ctx: ExecCtx, counter: u64) -> StdResult<Response> {
        self.counter.save(ctx.deps.storage, &counter)?;

        Ok(Response::new())
    }
}
```

Sylvia would add the field into the instantiation message, which now becomes this:

```rust
struct InstantiateMsg {
    counter: 64,
}
```

What is essential - the field in the `InstantiateMsg` gets the same name as the
function argument. Be careful using the typical Rust pattern to prefix it with
`_` to leave it unused. Unfortunately, to properly silence the unused warning here,
you need to add the `#[allow(unused)]` attribute before the argument or the whole
function.

Now let's add an execution message to the contract:

```rust
#[contract]
impl MyContract<'_> {
    #[msg(exec)]
    pub fn increment(&self, ctx: ExecCtx) -> StdResult<Response> {
        let counter = self.counter.load(ctx.deps.storage)?;
        self.counter.save(ctx.deps.storage, &(counter + 1))?;
        Ok(Response::new())
    }
}
```

Sylvia generated two message types from this:

```rust
enum ExecMsg {
    Increment {}
}

enum ContractExecMsg {
    MyContract(ExecMsg)
}
```

The `ExecMsg` is the primary one you may use to send messages to the contract.
The `ContractExecMsg` is only an additional abstraction layer that would matter
later when we define traits for our contract. For now, we just need to use it in
the entry point:

```rust
#[entry_point]
fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ContractExecMsg) -> StdResult<Response> {
    msg.dispatch(&CONTRACT, (deps, env, info))
}
```

One problem you might face now is that we use the `StdResult` for our contract,
but we often want to define the custom error type for our contracts - hopefully,
it is very easy to do:

```rust
```rust
use sylvia::cw_std::ensure;

#[contract(error=ContractError)]
impl MyContract<'_> {
    #[msg(exec)]
    pub fn increment(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let counter = self.counter.load(ctx.deps.storage)?;

        ensure!(counter < 10, ContractError::LimitReached);

        self.counter.save(ctx.deps.storage, &(counter + 1))?;
        Ok(Response::new())
    }
}
```

ContractError here is any error type you define for the contract - most typically
with the [thiserror](https://docs.rs/thiserror/1.0.40/thiserror/) crate. We also
need to update the error type in an entry points:

```rust
#[entry_point]
fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> Result<Response, ContractError> {
    msg.dispatch(&CONTRACT, (deps, env, info))
}

#[entry_point]
fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ContractExecMsg) -> Result<Response, ContractError> {
    msg.dispatch(&CONTRACT, (deps, env, info))
}
```

Finally, let's take a look at defining the query message:

```rust
use cosmwasm_schema::cw_serde;
use sylvia::types::QueryCtx;

#[cw_serde]
pub struct CounterResp {
    pub counter: u64,
}

#[contract(error=ContractError)]
impl MyContract<'_> {
    #[msg(query)]
    pub fn counter(&self, ctx: QueryCtx) -> StdResult<CounterResp> {
        self
            .counter
            .load(ctx.deps.storage)
            .map(|counter| CounterResp { counter })
    }
}
```

What you might notice - we can still use `StdResult` (so `StdError`) if we don't
need `ContractError` in a particular function. What is important is that the returned
result type has to implement `Into<ContractError>`, where `ContractError` is a contract
error type - it will all be commonized in the generated dispatching function (so
entry points have to return `ContractError` as its error variant).

Messages equivalent to execution messages are generated. Let's create an entry point:

```rust
use sylvia::cw_std::{Deps, Binary};

#[entry_point]
fn query(deps: Deps, env: Env, msg: ContractQueryMsg) -> Result<Binary, ContractError> {
    msg.dispatch(&CONTRACT, (deps, env))
}
```

## Interfaces

One of the fundamental ideas of Sylvia's framework is interfaces, allowing the
grouping of messages into their semantical groups. Let's define a Sylvia interface:

```rust
pub mod group {
    use super::*;
    use sylvia::interface;
    use sylvia::types::ExecCtx;
    use sylvia::cw_std::StdError;

    #[cw_serde]
    pub struct IsMemberResp {
        pub is_member: bool,
    }

    #[interface]
    pub trait Group {
        type Error: From<StdError>;

        #[msg(exec)]
        fn add_member(&self, ctx: ExecCtx, member: String) -> Result<Response, Self::Error>;

        #[msg(query)]
        fn is_member(&self, ctx: QueryCtx, member: String) -> Result<IsMemberResp, Self::Error>;
    }
}
```

Then we need to implement the trait on the contract type:

```rust
use sylvia::cw_std::{Empty, Addr};
use cw_storage_plus::{Map, Item};

pub struct MyContract<'a> {
    counter: Item<'a, u64>,
    // New field added - remember to initialize it in `new`
    members: Map<'a, &'a Addr, Empty>,
}

#[contract]
#[messages(group as Group)]
impl group::Group for MyContract<'_> {
    type Error = ContractError;

    #[msg(exec)]
    fn add_member(&self, ctx: ExecCtx, member: String) -> Result<Response, ContractError> {
        let member = ctx.deps.api.addr_validate(member)?;
        self.members.save(ctx.deps.storage, &member, &Empty {})?;
        Ok(Response::new())
    }

    #[msg(query)]
    fn is_member(&self, ctx: QueryCtx, member: String) -> Result<group::IsMemberResp, ContractError> {
        let is_member = self.members.has(ctx.deps.storage, &Addr::unchecked(&member));
        let resp = group::IsMemberResp {
            is_member,
        };

        Ok(resp)
    }
}

#[contract]
#[messages(group as Group)]
impl MyContract<'_> {
    // Nothing changed here
}
```

Here are a couple of things to talk about.

First, note that I defined the interface trait in its separate module with a name
matching the trait name. This is a requirement right now - Sylvia generates all
the messages and boilerplate in this module and will try to access them through
this module.

Then there is the `Error` type embedded in the trait - it is also needed there,
and the trait bound here has to be at least `From<StdError>`, as Sylvia might
generate code returning an `StdError` in deserialization/dispatching implementation.
The trait can be more strict - this is the minimum.

Another thing to remember is that the `#[msg(...)]` attributes become part of the
function signature - they must be the same for the trait and later implementation.

Finally, every implementation block has an additional
`#[messages(module as Identifier)]` attribute. Sylvia needs it to generate the dispatching
properly - there is the limitation that every macro has access only to its local
item. In particular - we cannot see all traits implemented by a type and their
implementation from the `#[contract]` crate.

To solve this issue, we put this `#[messages(...)]` attribute pointing to Sylvia
what is the module name where the interface is defined, and giving a unique name
for this interface (it would be used in generated code to provide proper enum variant).

The impl-block with trait implementation also contains the `#[messages]` attribute,
but only one - the one with info about the trait being implemented.

## Usage in external crates

What is important is the possibility of using generated code in the external code.
First, let's start with generating the documentation of the crate:

```sh
cargo doc --document-private-items --open
```

This generates and opens documentation of the crate, including all generated structures.
`--document-private-item` is optional, but it will generate documentation of not-public
modules which is sometimes useful.

Going through the doc, you will see that all messages are generated in their structs/traits
modules. To send messages to the contract, we can just use them:

```rust
use sylvia::cw_std::{WasmMsg, to_binary};

fn some_handler(my_contract_addr: String) -> StdResult<Response> {
    let msg = my_contract_crate::Execute::Increment {};
    let msg = WasmMsg::ExecMsg {
        contract_addr: my_contract_addr,
        msg: to_binary(&msg)?,
        funds: vec![],
    }

    let resp = Response::new()
        .add_message(msg);
    Ok(resp)
}
```

We can use messages from traits in a similar way:

```rust
let msg = my_contract_crate::group::QueryMsg::IsMember {
    member: addr,
};

let is_member: my_contract_crate::group::IsMemberResp =
    deps.querier.query_wasm_smart(my_contract_addr, &msg)?;
```

It is important not to confuse the generated `ContractExecMsg/ContractQueryMsg`
with `ExecMsg/QueryMsg` - the former is generated only for contract, not for interfaces,
and is not meant to use to send messages to the contract - their purpose is for proper
messages dispatching only, and should not be used besides the entry points.

## Using not implemented entry points

Sylvia is not yet implementing all the possible CosmWasm entry points, and even
when it will - it might happen that some will be added in the future, and Sylvia
would not align immediately. Hopefully, you can always use traditional entry points
for anything which is not implemented - for example, IBC calls. As an example, let's
see how to implement replies for messages:

```rust
use sylvia::cw_std::{DepsMut, Env, Reply, Response};

#[contract(error = ContractError)]
#[messages(group as Group)]
impl MyContract<'_> {
    fn reply(&self, deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
        todo!()
    }
    // Some items defined previously
}

#[entry_point]
fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    CONTRACT.reply(deps, env, reply)
}
```

It is important to create an entry function in the contract type - this way, it
gains access to all the state accessors defined on the type.

## Multitest

Sylvia also generates some helpers for testing contracts - it is hidden behind the
`mt` feature flag, which has to be enabled.

It is important to ensure no `mt` flag is set when the contract is built in `wasm`
target because of some dependencies it uses, which are not buildable on Wasm. My
recommendation is to add an additional `sylvia` entry with `mt` enabled in the
`dev-dependencies`, and also add the `mt` feature on your contract, which enables
mt utilities in other contract tests. An example `Cargo.toml`:

```rust
[package]
name = "my-contract"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
library = []
mt = ["sylvia/mt"]

[dependencies]
cosmwasm-schema = "1.2.5"
cosmwasm-std = "1.2.5"
cw-storage-plus = "1.0.1"
schemars = "0.8.12"
serde = "1.0.160"
sylvia = "0.3.2"
thiserror = "1.0.40"

[dev-dependencies]
sylvia = { path = "0.3.2", features = ["mt"] }
```

There would obviously be more dependencies - most probably `cw-strorage-plus`,
but this is just to show how I enable the `mt` flag. With that, we can use mt
utils in the contract:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sylvia::multitest::App;

    #[test]
    fn counter_test() {
        let app = App::default();

        let owner = "owner";

        let code_id = contract::CodeId::store_code(&app);

        let contract = code_id.instantiate(3)
            .with_label("My contract")
            .call(owner)
            .unwrap();

        let counter = contract.counter().unwrap();
        assert_eq!(counter, contract::CounterResp { counter: 3});

        contract.increment().call(owner).unwrap();

        let counter = contract.counter().unwrap();
        assert_eq!(counter, contract::CounterResp { counter: 4});
    }
}
```

First of all, note the `contract` module I am using here - it is a slight change
that doesn't match the previous code - I assume here that all the contract code
sits in the `contract` module to make sure it is clear where the used type lies.
So if I use `contract::something`, it is `something` in the module of the original
contract (most probably sylvia-generated).

First of all - we do not use `cw-multi-test` app directly. Instead we use the `sylvia`
wrapper over it. It contains the original multi-test App internally, but it does
it in an internally-mutable manner which makes it possible to avoid passing it
everywhere around. It adds some overhead, but it should not matter for testing code.

We are first using the `CodeId` type generated for every single Sylvia contract
separately. Its purpose is to abstract storing the contract in the blockchain. It
makes sure to create the contract object and pass it to the multitest.

A contract's `CodeId` type has one particularly interesting function - the `instantiate`,
which calls an instantiation function. It takes the same arguments as an instantiation
function in the contract, except for the context that Sylvia's utilities would provide.

The function doesn't instantiate contract immediately - instead, it returns what
is called `InstantiationProxy`. We decided that we don't want to force users to set
all the metadata - admin, label, and funds to send with every instantiation call,
as in the vast majority of cases, they are irrelevant. Instead, the
`InstantiationProxy` provides `with_label`, `with_funds`, and `with_amin` functions,
which set those meta fields in the builder pattern style.

When the instantiation is ready, we call the `call` function, passing the message
sender - we could add another `with_sender` function, but we decided that as the
sender has to be passed every single time, we can save some keystrokes on that.

The thing is similar when it comes to execution messages. The biggest difference
is that we don't call it on the `CodeId`, but on instantiated contracts instead.
We also have fewer fields to set on that - the proxy for execution provides only
the `with_funds` function.

All the instantiation and execution functions return the
`Result<cw_multi_test::AppResponse, ContractError>` type, where `ContractError`
is an error type of the contract.

## Interface items in multitest

Because of implementation restrictions, calling methods from the contract interface
looks slightly different:

```rust
#[test]
fn member_test() {
    let app = App::default();

    let owner = "owner";
    let member = "john";

    let code_id = contract::multitest_utils::CodeId::store_code(&app);

    let contract = code_id.instantiate(0)
        .with_label("My contract")
        .call(owner);

    contract
        .group()
        .add_member(member.to_owned())
        .call(owner);

    let resp = contract
        .group_proxy()
        .is_member(member.to_owned())

    assert_eq!(resp, group::IsMemberResp { is_member: true });
}
```

Note an additional `group_proxy()` call for executions and queries - it returns an
extra proxy wrapper that would send the messages from a particular interface.

## Generating schema

Sylvia is designed to generate all the code which cosmwasm-schema relies on - this
makes it very easy to generate schema for the contract. Just add a `bin/schema.rs`
module, which would be recognized as a binary, and add a simple main function there:

```rust
use cosmwasm_schema::write_api;

use my_contract_crate::contract::{ContractExecMsg, ContractQueryMsg, InstantiateMsg};

fn main() {
    write_api! {
        instantiate: InstantiateMsg,
        execute: ContractExecMsg,
        query: ContractQueryMsg,
    }
}
```

Unfortunately, because of [a bug](https://github.com/CosmWasm/ts-codegen/issues/103)
in the `ts-codegen`, schemas for Sylvia contracts are  not properly interpreted there.
However, we are working on how to solve this issue regardless of the `ts-codegen`
implementation.

## Road map

Sylvia is in the adoption stage right now, but we are still working on more and more
features for you. Here is a rough roadmap for the incoming months:

* Entry points generation - You will no longer need to write the entry points boilerplate
  yourself! Sylvia can do it for you.
* Queriers helpers - As queries are not using any actor-model flow, we want to be
  able just to query the contract with some nice helpers - and we will. Querying
  external contracts would be as simple as calling them in multi-tests!
* Replies - Sylvia still needs support for essential CosmWasm messages, which are
  replies. We want to make them smart, so expressing the correlation between send
  message end executed handler is more direct and not hidden in the reply dispatcher.
* Migrations - Another important message we don't support, but the reason is similar
  to replies - we want them to be smart. We want to give you a nice way to provide
  upgrading Api for your contract, which would take care of its versioning.
* IBC - we want to give you a nice IBC Api too! However, expect it to be a
  while - we must first understand the best patterns here.
* Better tooling support - The biggest issue of Sylvia is that code it generates
  is not trivial, and not all the tooling handles it well. We are working on improving
  user experience in that regard.
