# Sylvia Framework

Sylvia is the old name meaning Spirit of The Wood.

Sylvia is the Roman goddess of the forest.

Sylvia is also a framework created to give you the abstraction-focused and
scalable solution for building your CosmWasm Smart Contracts. Find your way
into the forest of Cosmos ecosystem. We provide you with the toolset, so instead
of focusing on the raw structure of your contract, you can create it in proper
and idiomatic Rust and then just let cargo make sure that they are sound.

Learn more about `sylvia` in [the book](https://cosmwasm.github.io/sylvia-book/index.html)

## Sylvia contract template

The Sylvia template streamlines the development of CosmWasm smart contracts by providing a project scaffold that adheres to best practices and leverages the Sylvia framework's powerful features. It's designed to help developers focus more on their contract's business logic rather than boilerplate code.

Learn more here: [Sylvia Template on GitHub](https://github.com/CosmWasm/sylvia-template)

## The approach

[CosmWasm](https://cosmwasm.com/) ecosystem core provides the base building
blocks for smart contracts - the
[cosmwasm-std](https://crates.io/crates/cosmwasm-std) for basic CW bindings, the
[cw-storage-plus](https://crates.io/crates/cw-storage-plus) for easier state management,
and the [cw-multi-test](https://crates.io/crates/cw-multi-test) for testing them.
Sylvia framework is built on top of them, so for creating contracts, you don't
have to think about message structure, how their API is (de)serialized, or how
to handle message dispatching. Instead, the API of your contract is a set of
traits you implement on your contract type. The framework generates things like entry
point structures, functions dispatching the messages, or even helpers for multitest.
It allows for better control of interfaces, including validating their completeness
in compile time.

## Code generation

Sylvia macros generate code in the `sv` module. This means that every `contract` and
`interface` macro call must be made in a separate module to avoid collisions between
the generated modules.

## Contract type

In Sylvia, we define our contracts as structures:

```rust
use cw_storage_plus::Item;
use cosmwasm_schema::cw_serde;
use sylvia::types::QueryCtx;
use sylvia::cw_std::ensure;


/// Our new contract type.
///
struct MyContract<'a> {
    pub counter: Item<'a, u64>,
}


/// Response type returned by the
/// query method.
/// 
#[cw_serde]
pub struct CounterResp {
    pub counter: u64,
}

#[entry_points]
#[contract]
#[sv::error(ContractError)]
impl MyContract<'_> {
    pub fn new() -> Self {
        Self {
            counter: Item::new("counter")
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(&self, ctx: InstantiateCtx, counter: u64) -> StdResult<Response> {
        self.counter.save(ctx.deps.storage, &counter)?;
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn increment(&self, ctx: ExecCtx) -> Result<Response, ContractError> {
        let counter = self.counter.load(ctx.deps.storage)?;
        ensure!(counter < 10, ContractError::LimitReached);
        self.counter.save(ctx.deps.storage, &(counter + 1))?;
        Ok(Response::new())
    }

    #[sv::msg(query)]
    pub fn counter(&self, ctx: QueryCtx) -> StdResult<CounterResp> {
        self
            .counter
            .load(ctx.deps.storage)
            .map(|counter| CounterResp { counter })
    }
}
```

Sylvia will generate the following new structures:

```rust
pub mod sv {
    use super::*;

    struct InstantiateMsg {
        counter: u64,
    }

    enum ExecMsg {
        Increment {}
    }

    enum ContractExecMsg {
        MyContract(ExecMsg)
    }

    enum QueryMsg {
        Counter {}
    }

    enum ContractQueryMsg {
        MyContract(QueryMsg)
    }

    // [...]
}

pub mod entry_points {
    use super::*;

    #[sylvia::cw_std::entry_point]
    pub fn instantiate(
        deps: sylvia::cw_std::DepsMut,
        env: sylvia::cw_std::Env,
        info: sylvia::cw_std::MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<sylvia::cw_std::Response, StdError> {
        msg.dispatch(&MyContract::new(), (deps, env, info))
            .map_err(Into::into)
    }

    // [...]
}
```

`entry_points` macro generates `instantiate`, `execute`, `query` and `sudo` entry points.
All those methods call `dispatch` on the msg received and run proper logic defined for the sent
variant of the message.

What is essential - the field in the `InstantiateMsg` (and other messages) gets the same name as the
function argument.

The `ExecMsg` is the primary one you may use to send messages to the contract.
The `ContractExecMsg` is only an additional abstraction layer that would matter
later when we define traits for our contract.
Thanks to the `entry_point` macro it is already being used in the generated entry point and we don't
have to do it manually.

What you might notice - we can still use `StdResult` (so `StdError`) if we don't
need `ContractError` in a particular function. What is important is that the returned
result type has to implement `Into<ContractError>`, where `ContractError` is a contract
error type - it will all be commonized in the generated dispatching function (so
entry points have to return `ContractError` as its error variant).


## Interfaces

One of the fundamental ideas of the Sylvia framework is the interface, allowing the
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

        #[sv::msg(exec)]
        fn add_member(&self, ctx: ExecCtx, member: String) -> Result<Response, Self::Error>;

        #[sv::msg(query)]
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

impl group::Group for MyContract<'_> {
    type Error = ContractError;

    fn add_member(&self, ctx: ExecCtx, member: String) -> Result<Response, ContractError> {
        let member = ctx.deps.api.addr_validate(&member)?;
        self.members.save(ctx.deps.storage, &member, &Empty {})?;
        Ok(Response::new())
    }

    fn is_member(&self, ctx: QueryCtx, member: String) -> Result<group::IsMemberResp, ContractError> {
        let is_member = self.members.has(ctx.deps.storage, &Addr::unchecked(&member));
        let resp = group::IsMemberResp {
            is_member,
        };

        Ok(resp)
    }
}

#[contract]
#[sv::messages(group as Group)]
impl MyContract<'_> {
    // Nothing changed here
}
```

First, note that I defined the interface trait in its separate module with a name
matching the trait name, but written "snake_case" instead of CamelCase. Here I have the
`group` module for the `Group` trait, but the `CrossStaking` trait should be placed
in its own `cross_staking` module (note the underscore). This is a requirement right
now - Sylvia generates all the messages and boilerplate in this module and will try
to access them through this module. If the interface's name is a camel-case
version of the last module path's segment, the `as InterfaceName` can be omitted.
F.e. `#[sv::messages(cw1 as Cw1)]` can be reduced to `#[sv::messages(cw1)]`

Then there is the `Error` type embedded in the trait - it is also needed there,
and the trait bound here has to be at least `From<StdError>`, as Sylvia might
generate code returning the `StdError` in deserialization/dispatching implementation.
The trait can be more strict - this is the minimum.

Finally, the implementation block has an additional
`#[sv::messages(module as Identifier)]` attribute. Sylvia needs it to generate the dispatching
properly - there is the limitation that every macro has access only to its local
scope. In particular - we cannot see all traits implemented by a type and their
implementation from the `#[contract]` crate.

To solve this issue, we put this `#[sv::messages(...)]` attribute pointing to Sylvia
what is the module name where the interface is defined, and giving a unique name
for this interface (it would be used in generated code to provide proper enum variant).

## Macro attributes

```rust
struct MyMsg;
impl CustomMsg for MyMsg {}

struct MyQuery;
impl CustomQuery for MyMsg {}

#[entry_point]
#[contract]
#[sv::error(ContractError)]
#[sv::messages(interface as Interface)]
#[sv::messages(interface as InterfaceWithCustomType: custom(msg, query))]
#[sv::custom(msg=MyMsg, query=MyQuery)]
#[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::SudoMsg))]
impl MyContract {
    // ...
}
```

 * `sv::error` is used by both `contract` and `entry_point` macros. It is necessary in case a custom
   error is being used by your contract. If omitted generated code will use `StdError`.

 * `sv::messages` is the attribute for the `contract` macro. Its purpose is to inform Sylvia
   about interfaces implemented for the contract. If the implemented interface does not use a
   default `Empty` message response for query and/or exec then the `: custom(query)`,
   `: custom(msg)` or `: custom(msg, query)` should be indicated.

 * `sv::override_entry_point` - refer to the `Overriding entry points` section.

 * `sv::custom` allows to define CustomMsg and CustomQuery for the contract. By default generated code
    will return `Response<Empty>` and will use `Deps<Empty>` and `DepsMut<Empty>`.


## Usage in external crates

What is important is the possibility of using generated code in the external code.
First, let's start with generating the documentation of the crate:

```sh
cargo doc --document-private-items --open
```

This generates and opens documentation of the crate, including all generated structures.
`--document-private-item` is optional, but it will generate documentation of non-public
modules which is sometimes useful.

Going through the doc, you will see that all messages are generated in their structs/traits
modules. To send messages to the contract, we can just use them:

```rust
use sylvia::cw_std::{WasmMsg, to_json_binary};

fn some_handler(my_contract_addr: String) -> StdResult<Response> {
    let msg = my_contract_crate::sv::ExecMsg::Increment {};
    let msg = WasmMsg::ExecMsg {
        contract_addr: my_contract_addr,
        msg: to_json_binary(&msg)?,
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
and is not meant to be used to send messages to the contract - their purpose is for proper
messages dispatching only, and should not be used besides the entry points.


## Query helpers

To make querying more user-friendly `Sylvia` provides users with `sylvia::types::BoundQuerier` and 
`sylvia::types::Remote` helpers. The latter is meant to store the address of some remote contract.
For each query method in the contract, Sylvia will add a method in a generated `sv::Querier` trait.
The `sv::Querier` is then implemented for `sylvia::types::BoundQuerier` so the user can call the method.

Let's modify the query from the previous paragraph. Currently, it will look as follows:

```rust
let is_member = Remote::<OtherContractType>::new(remote_addr)
    .querier(&ctx.deps.querier)
    .is_member(addr)?;
```

Your contract might communicate with some other contract regularly.
In such a case you might want to store it as a field in your Contract:

```rust
pub struct MyContract<'a> {
    counter: Item<'a, u64>,
    members: Map<'a, &'a Addr, Empty>,
    remote: Item<'a, Remote<'static, OtherContractType>>,
}

#[sv::msg(exec)]
pub fn evaluate_member(&self, ctx: ExecCtx, ...) -> StdResult<Response> {
    let is_member = self
        .remote
        .load(ctx.deps.storage)?
        .querier(&ctx.deps.querier)
        .is_member(addr)?;
}
```

## Using unsupported entry points

If there's a need for an entry point that is not implemented in Sylvia, you can implement
it manually using the `#[entry_point]` macro. As an example, let's see how to implement
replies for messages:

```rust
use sylvia::cw_std::{DepsMut, Env, Reply, Response};

#[contract]
#[entry_point]
#[sv::error(ContractError)]
#[sv::messages(group as Group)]
impl MyContract<'_> {
    fn reply(&self, deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
        todo!()
    }
    // [...]
}

#[entry_point]
fn reply(deps: DepsMut, env: Env, reply: Reply) -> Result<Response, ContractError> {
    &MyContract::new().reply(deps, env, reply)
}
```

It is important to create an entry function in the contract type - this way, it
gains access to all the state accessors defined on the type.

## Overriding entry points

There is a way to override an entry point or to add a custom-defined one.
Let's consider the following code:

```rust
#[cw_serde]
pub enum UserExecMsg {
    IncreaseByOne {},
}

pub fn increase_by_one(ctx: ExecCtx) -> StdResult<Response> {
    crate::COUNTER.update(ctx.deps.storage, |count| -> Result<u32, StdError> {
        Ok(count + 1)
    })?;
    Ok(Response::new())
}

#[cw_serde]
pub enum CustomExecMsg {
    ContractExec(crate::ContractExecMsg),
    CustomExec(UserExecMsg),
}

impl CustomExecMsg {
    pub fn dispatch(self, ctx: (DepsMut, Env, MessageInfo)) -> StdResult<Response> {
        match self {
            CustomExecMsg::ContractExec(msg) => {
                msg.dispatch(&crate::contract::Contract::new(), ctx)
            }
            CustomExecMsg::CustomExec(_) => increase_by_one(ctx.into()),
        }
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: CustomExecMsg,
) -> StdResult<Response> {
    msg.dispatch((deps, env, info))
}
```

It is possible to define a custom `exec` message that will dispatch over one generated
by your contract and one defined by you. To use this custom entry point with `contract` macro
you can add the `sv::override_entry_point(...)` attribute.

```rust    
#[contract]
#[sv::override_entry_point(exec=crate::entry_points::execute(crate::exec::CustomExecMsg))]
#[sv::override_entry_point(sudo=crate::entry_points::sudo(crate::SudoMsg))]
impl Contract {
    // ...
}
```

It is possible to override all message types like that. Next to the entry point path, you will
also have to provide the type of your custom message. It is required to deserialize the message
in the `multitest helpers`.

## Multitest

Sylvia also generates some helpers for testing contracts - it is hidden behind the
`mt` feature flag, which has to be enabled.

It is important to ensure no `mt` flag is set when the contract is built in `wasm`
target because of some dependencies it uses, which are not buildable on Wasm. The
recommendation is to add an extra `sylvia` entry with `mt` enabled in the
`dev-dependencies`, and also add the `mt` feature on your contract, which enables
mt utilities in other contract tests. An example `Cargo.toml`:

```toml
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
sylvia = "0.10.0"

# [...]

[dev-dependencies]
sylvia = { version = "0.10.0", features = ["mt"] }
```

And the example code:

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

Note the `contract` module I am using here - it is a slight change
that doesn't match the previous code - I assume here that all the contract code
sits in the `contract` module to make sure it is clear where the used type lies.
So if I use `contract::something`, it is `something` in the module of the original
contract (most probably Sylvia-generated).

First of all - we do not use `cw-multi-test` app directly. Instead, we use the `sylvia`
wrapper over it. It contains the original multi-test App internally, but it does
it in an internally mutable manner which makes it possible to avoid passing it
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

Trait declaring all the interface methods is directly implemented on
the contracts Proxy type.

```rust
use contract::mt::Group;

#[test]
fn member_test() {
    let app = App::default();

    let owner = "owner";
    let member = "john";

    let code_id = contract::mt::CodeId::store_code(&app);

    let contract = code_id.instantiate(0)
        .with_label("My contract")
        .call(owner);

    contract
        .add_member(member.to_owned())
        .call(owner);

    let resp = contract
        .is_member(member.to_owned())

    assert_eq!(resp, group::IsMemberResp { is_member: true });
}
```

## Generics

### Interface

Defining associated types on an interface is as simple as defining them on a regular trait.

```rust
#[interface]
pub trait Generic {
    type Error: From<StdError>;
    type ExecParam: CustomMsg;
    type QueryParam: CustomMsg;
    type RetType: CustomMsg;

    #[sv::msg(exec)]
    fn generic_exec(
        &self,
        ctx: ExecCtx,
        msgs: Vec<CosmosMsg<Self::ExecParam>>,
    ) -> Result<Response, Self::Error>;

    #[sv::msg(query)]
    fn generic_query(&self, ctx: QueryCtx, param: Self::QueryParam) -> Result<Self::RetType, Self::Error>;
}
```

### Generic contract

Generics in a contract might be either used as generic field types or as generic parameters of return
types in the messages. When Sylvia generates the messages' enums, only generics used in respective methods
will be part of a given generated message type.


Example of usage:
```rust
pub struct GenericContract<
    InstantiateParam,
    ExecParam,
    FieldType,
> {
    _field: Item<'static, FieldType>,
    _phantom: std::marker::PhantomData<(
        InstantiateParam,
        ExecParam,
    )>,
}

#[contract]
impl<InstantiateParam, ExecParam, FieldType>
    GenericContract<InstantiateParam, ExecParam, FieldType>
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    FieldType: 'static,
{
    pub const fn new() -> Self {
        Self {
            _field: Item::new("field"),
            _phantom: std::marker::PhantomData,
        }
    }

    #[sv::msg(instantiate)]
    pub fn instantiate(
        &self,
        _ctx: InstantiateCtx,
        _msg: InstantiateParam,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    #[sv::msg(exec)]
    pub fn contract_execute(
        &self,
        _ctx: ExecCtx,
        _msg: ExecParam,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }
}
```

### Implement interface

```rust
impl<InstantiateParam, ExecParam, FieldType>
    Generic
    for crate::contract::GenericContract<
        InstantiateParam,
        ExecParam,
        FieldType,
    >
{
    type Error = StdError;
    type ExecParam = ExecParam;
    type QueryParam: SvCustomMsg;
    type RetType = SvCustomMsg;

    fn generic_exec(
        &self,
        _ctx: ExecCtx,
        _msgs: Vec<CosmosMsg<Self::ExecParam>>,
    ) -> StdResult<Response> {
        Ok(Response::new())
    }

    fn generic_query(
        &self,
        _ctx: QueryCtx,
        _msg: Self::QueryParam,
    ) -> StdResult<Self::RetType> {
        Ok(SvCustomMsg {})
    }
}
```

Now we have to inform Sylvia that the interface implemented for the contract has associated types.
We have to list those types (generics or concrete) next to the interface in the `#[sv::messages]`
attribute:

```rust
#[contract]
#[sv::messages(generic<ExecParam, SvCustomMsg, SvCustomMsg> as Generic)]
impl<InstantiateParam, ExecParam, FieldType>
    GenericContract<InstantiateParam, ExecParam, FieldType>
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    FieldType: 'static,
{
    ...
}
```

### Generics in entry_points

Entry points have to be generated with concrete types. Using the `entry_points` macro
on the generic contract we have to specify the types that have to be used.
We do that with `entry_points(generics<..>)`:

```rust
#[cfg_attr(not(feature = "library"), entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg>))]
#[contract]
impl<InstantiateParam, ExecParam, FieldType>
    GenericContract<InstantiateParam, ExecParam, FieldType>
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    FieldType: 'static,
{
    ...
}
```

The contract might define a generic type in place of a custom message and query.
In such case we have to inform `entry_points` macro using `custom`:

```rust
#[cfg_attr(not(feature = "library"), entry_points(generics<SvCustomMsg, SvCustomMsg, SvCustomMsg>, custom(msg=SvCustomMsg, query=SvCustomQuery))]
#[contract]
#[sv::custom(msg=MsgT, query=QueryT)]
impl<InstantiateParam, ExecParam, FieldType, MsgT, QueryT>
    GenericContract<InstantiateParam, ExecParam, FieldType, MsgT, QueryT>
where
    for<'msg_de> InstantiateParam: CustomMsg + Deserialize<'msg_de> + 'msg_de,
    ExecParam: CustomMsg + DeserializeOwned + 'static,
    FieldType: 'static,
{
    ...
}
```


## Generating schema

Sylvia is designed to generate all the code that `cosmwasm-schema` relies on - this
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

## Road map

Sylvia is in the adoption stage right now, but we are still working on more and more
features for you. Here is a rough roadmap for the coming months:

- Replies - Sylvia still needs support for essential CosmWasm messages, which are
  replies. We want to make them smart, so expressing the correlation between the sent
  message and the executed handler is more direct and not hidden in the reply dispatcher.
- Migrations - Another important message we don't support, but the reason is similar
  to replies - we want them to be smart. We want to give you a nice way to provide
  upgrading Api for your contract, which would take care of its versioning.
- IBC - we want to give you a nice IBC Api too! However, expect it to be a
  while - we must first understand the best patterns here.
- Better tooling support - The biggest Sylvia issue is that the code it generates
  is not trivial, and not all the tooling handles it well. We are working on improving
  user experience in that regard.

## Troubleshooting

For more descriptive error messages, consider using the nightly toolchain (add `+nightly`
argument for cargo)

- Missing messages from an interface on your contract - You may be missing the
  `#[sv::messages(interface as Interface)]` attribute.
