# Migrating Contracts

This guide explains what is needed to upgrade contracts when migrating over major releases of `sylvia`. Note that you can also view the [complete CHANGELOG](https://github.com/CosmWasm/sylvia/blob/main/CHANGELOG.md) to understand the differences.


## 1.0.2 ->


### Generics in `sv::messages` not required
```diff
-#[contract]
-#[sv::messages(generic<SomeType1, SomeType2, SomeType3> as Generic
-impl Contract {}
+#[contract]
+#[sv::messages(generic as Generic)]
+impl Contract {}
```

This change is optional, since the generics are still accepted by the parser. Though they are
ignored in the further process.


### CodeId generic over the Contract type
```diff
-let code_id: CodeId<
-    SvCustomMsg,
-    SvCustomMsg,
-    _,
-> = CodeId::store_code(&app);
+let code_id: CodeId<
+    GenericContract<
+        SvCustomMsg,
+        SvCustomMsg,
+    >,
+    _,
+> = CodeId::store_code(&app);
```

### Lifetime ellision in a contract's impl block not supported
```diff
-#[contract]
-impl Cw1SubkeysContract<'_> {
-    // [...]
-}
+#[contract]
+impl<'a> Cw1SubkeysContract<'a> {
+    // [...]
+}
```


## 0.9.3 -> 0.10.0

## Multitest proxy

Since `0.10.0` Sylvia won't generate multitest Proxy types for the `Interface` macro call. Instead, all the methods from interfaces are directly implemented on the contract's multitest proxy.
To use methods from an implemented interface like before, the user has to import the multitest trait from the module in which the interface is implemented.

```diff
-let resp = contract
-   .cw1_proxy()
-   .can_execute()
-   .unwrap();
+let resp = contract
+   .can_execute()
+   .unwrap();
```

## Associated types in generics

`Sylvia` interface is meant to be implemented on contract only once. Because of that, we decided to remove support for generics in interfaces.
Instead, when migrating from `0.9.3` to `0.10.0`, the user must replace generics with associated types.

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
