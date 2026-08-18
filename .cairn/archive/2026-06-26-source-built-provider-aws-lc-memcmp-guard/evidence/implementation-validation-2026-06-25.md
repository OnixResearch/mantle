# Implementation validation: AWS-LC memcmp guard handling

Date: 2026-06-25
Change: `source-built-provider-aws-lc-memcmp-guard`

## Scope

This evidence covers implementation tasks I3/I4 and focused verification tasks V1/V2 for `r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]`.

Implemented files:

- `src/source_toolchain_closure.rs`
  - added `ReceiptBoundCCompilerRoute`
  - added `select_receipt_bound_c_compiler_route(...)`
  - added `validate_receipt_bound_c_compiler_route(...)`
  - added fail-closed ambiguous-route classification
- `src/cargo_free_self_build.rs`
  - passes selected receipt-bound C compiler route to `rust-plan` through a hidden `--source-built-c-compiler-route-json` argument and the compatibility `MANTLE_SOURCE_BUILT_C_COMPILER_ROUTE` env bridge
  - makes receipt-bound `cc` alias resolve to the selected closure route
  - rejects ambiguous/undeclared route selection before provider proof execution
- `src/main.rs`
  - accepts the hidden route argument for provider proof execution and installs a one-shot route override for the rust-plan executor
- `src/rust_plan.rs`
  - maps AWS-LC GCC PR95189 `memcmp` guard diagnostics to stable `unsupported-compiler-guard`
  - records selected compiler route on successful build-script metadata receipts
  - keeps ambient `CC`, `CFLAGS`, `HOST`, and `TARGET` out of build-script child execution env

## Focused Rust tests

### Route selection and route JSON validation

Command (pueue task 183):

```text
cargo test -p mantle --bin mantle c_compiler_route
```

Result:

```text
running 5 tests
test rust_plan::tests::source_built_c_compiler_route_json_validates_receipt_bound_identity ... ok
test rust_plan::tests::source_built_c_compiler_route_json_rejects_non_compiler_route ... ok
test source_toolchain_closure::tests::c_compiler_route_selection_prefers_single_clang_route ... ok
test source_toolchain_closure::tests::c_compiler_route_selection_records_receipt_bound_identity ... ok
test source_toolchain_closure::tests::c_compiler_route_selection_rejects_ambiguous_non_clang_routes ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 863 filtered out; finished in 0.00s
```

### Stable compiler-guard blocker and bypass-env rejection

Command (pueue task 190):

```text
cargo test -p mantle --bin mantle compiler_guard
```

Result:

```text
running 3 tests
test rust_plan::tests::rust_topology_child_env_rejects_ambient_compiler_guard_bypass_env ... ok
test rust_plan::tests::aws_lc_memcmp_guard_failure_becomes_stable_compiler_guard_blocker ... ok
test rust_plan::tests::build_script_child_env_does_not_forward_compiler_guard_bypass_env ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 865 filtered out; finished in 0.00s
```

### Selected compiler route receipt recording

Command (current rerun output captured in session):

```text
cargo test -p mantle --bin mantle selected_compiler
```

Result:

```text
running 1 test
test rust_plan::tests::build_script_metadata_success_receipt_records_selected_compiler_route ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 867 filtered out; finished in 0.00s
```

### Receipt-bound PATH aliases

Command (pueue task 199):

```text
cargo test -p mantle --bin mantle receipt_bound_path_aliases
```

Result:

```text
running 6 tests
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_unsafe_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_member_name_conflict ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_linker_for_collect2 ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_target_prefixed_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 862 filtered out; finished in 0.02s
```

## Interpretation

- Positive route tests prove a single receipt-bound route is recorded, a single receipt-bound Clang route is preferred when multiple C compiler routes exist, and successful metadata receipts can carry the selected route.
- Negative route/bypass tests prove ambiguous non-Clang routes fail closed, non-C-compiler route JSON is rejected, and ambient `CC`/`CFLAGS`/`HOST`/`TARGET` values are not forwarded as guard bypasses.
- This does not claim the current source-built provider fixed point succeeds; V3 owns the real provider proof rerun.
