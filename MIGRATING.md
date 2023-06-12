# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over major releases of `sylvia`. Note that you can also view the [complete CHANGELOG](https://github.com/CosmWasm/sylvia/blob/main/CHANGELOG.md) to understand the differences.

## 0.5.0 -> Unreleased

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

`InstantiateCtx` and `ExecCtx` could be previosly used interchangeably. They are currently separate
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

`module` attr for macro contract should now point to your contract implemntation
