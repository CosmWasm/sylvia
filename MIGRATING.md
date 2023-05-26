# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over major releases of `sylvia`. Note that you can also view the [complete CHANGELOG](https://github.com/CosmWasm/sylvia/blob/main/CHANGELOG.md) to understand the differences.

## 0.4.2 -> Unreleased

- `module` attr for macro contract should now point to your contract implemntation

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
