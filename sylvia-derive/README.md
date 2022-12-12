# Sylvia Framework

Sylvia is the old name meaning Spirit of The Wood.

Sylvia is the Roman goddess of the forest.

Sylvia is also a framework created to give you the abstraction-focused and scalable solution for
building your CosmWasm Smart Contracts. Find your way into the forest of Cosmos ecosystem. We
provide you toolset, so instead of focusing on the raw structure of your contract, you can create
them in proper and idiomatic Rust and then just let cargo make sure that they are sound.

This crate is just a backbones implementation. You should always use [sylvia](https://crates.io/crates/sylvia)
instead of this crate.

## The approach

[CosmWasm](https://cosmwasm.com/) ecosystem core provides the base building blocks for smart
contracts - the [cosmwasm-std](https://crates.io/crates/cosmwasm-std) for basic CW bindings,
the [cw-storage-plus](https://crates.io/crates/cw-storage-plus) for easier state management,
and the [cw-multi-test](https://crates.io/crates/cw-multi-test) for testing them. Sylvia framework
is built on top of them, so creating contracts, you don't need to think about message structure,
how their API is (de)serialized, or how to handle message dispatching. Instead, the API of your
contract is a set of traits you implement on your SC type. The framework generates things like
entry point structures, functions dispatching the messages, or even helpers for multitest. It
allows for better control of interfaces, including validating their completeness in compile time.

Also, as a side effect, as Sylvia has all the knowledge about the contract API structure,
it can generate many helpers - utilities for multitests or even queriers.

## The state

Right now, Sylvia is ready to implement simple smart contracts. You can implement Instantiation,
Execution, Query, and Migrate messages quickly. Sudo, and reply messages are missing right now.
Still, all of those can be covered by simply implementing their entry points and dispatching manually,
so for basic functionality, Sylvia also can always be used. Sylvia already supports overriding the
chain custom messages, so it is possible to use it with custom blockchains - except for handling custom
queries and sudo messages. All messages can be generic, so creating a contract eligible to work
across several different CosmWasm blockchains is even possible.

For now, only messages dispatching are generated - queriers utilities and MT helpers are on the TODO list, so expect them soon.
