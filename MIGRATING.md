# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over major releases of `sylvia`. Note that you can also view the [complete CHANGELOG](https://github.com/CosmWasm/sylvia/blob/main/CHANGELOG.md) to understand the differences.

## 0.8.1 -> 0.9.0

### `sv` module

Sylvia from now on will generate all the code in the `sv` module. This means that you will need to update your imports to use the new module.

### Implementing non-custom interface on custom contract

In Sylvia `0.8.x` there was missing check for the `,` in `#[messages(cw1 as Cw1: custom(msg query))]`.
Since `0.9.0` Sylvia expects user to split `msg` and `query` with `,` as such `#[messages(cw1 as Cw1: custom(msg, query))]`.

## 0.5.0 -> 0.6.0

### Context distinction

```
error[E0308]: mismatched types
  --> contracts/cw1-subkeys/src/contract.rs:56:49
   |
56 |         let result = self.whitelist.instantiate(ctx.branch(), admins, mutable)?;
   |                                     ----------- ^^^^^^^^^^^^ expected `ExecCtx<'_>`, found `InstantiateCtx<'_>`
   |                                     |
   |                                     arguments to this method are incorrect
   |
   = note: expected struct `ExecCtx<'_>`
              found struct `InstantiateCtx<'_>`
```

`InstantiateCtx` and `ExecCtx` could be previously used interchangeably. They are currently separate
  types. The same applies to `ReplyCtx`/`MigrateCtx` pair.

## 0.4.2 -> 0.5.0

### `module` attribute repurpose

```diff
-#[contract(module=some_contract)]
-impl SomeContract {
-    ...
-}
+mod some_contract {
+   #[contract]
+   impl SomeContract {
+       ...
+   }
+}
```

`module` attr for macro contract should now point to your contract implementation
