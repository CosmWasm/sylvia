# CHANGELOG

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2022-10-18

This is the first documented and supported implementation. It provides
macro to generate messsages for interfaces and contracts.

Some main points:

- Support for instantiate, execute, query, migrate and reply messages.
- Ability to implement multiple interfaces on contract.
- Mechanism of detecting overlapping of messages.
- Dispatch mechanism simplyfing entry points creation.
- Support for schema generation.