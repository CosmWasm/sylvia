# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## \[0.3.1\] - 2023-03-03

- Slight improvement the invalid message received error

## \[0.3.0\] - 2023-02-01

- Interfaces moved to separate directory to avoid errors on workspace optimizer
- `mt` feature added. Enabling it will:
  - generate `cw_multi_test::Contract` impl on a contract
  - generate Proxy to simplify writting tests
- Example of usage of new test framework
- Port of `cw20` contract on `sylvia` framework
- Default error type on contract is now `cosmwasm_std::StdError`
- Reexported `schemars`

## \[0.2.2\] - 2022-12-13

- Fix: Generate Migrate as struct
- Cw20 implementation in sylvia
- Removed `#[msg(reply)]`

## \[0.2.1\] - 2022-10-19

This is the first documented and supported implementation. It provides
macro to generate messsages for interfaces and contracts.

Some main points:

- Support for instantiate, execute, query, migrate and reply messages.
- Ability to implement multiple interfaces on contract.
- Mechanism of detecting overlapping of messages.
- Dispatch mechanism simplyfing entry points creation.
- Support for schema generation.
