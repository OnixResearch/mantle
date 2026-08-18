# Target alias focused validation

Task-ID: V1
Covers: rust_package_planning.source_built_toolchain_closure.target_aliases

## Result

Passed.

## Commands

Pueue task 20:

```text
rustfmt src/cargo_free_self_build.rs
rustfmt --check src/cargo_free_self_build.rs src/source_toolchain_closure.rs
git diff --check
cargo test -p mantle --bin mantle cargo_free_self_build::tests::receipt_bound_path_aliases_ -- --nocapture
cargo test -p mantle --bin mantle cargo_free_self_build::tests::receipt_bound_enforcement_ -- --nocapture
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Visible result excerpt from task 20:

```text
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 742 filtered out; finished in 0.04s

{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

Pueue task 21 reran the alias-focused tests with complete visible output:

```text
cargo test -p mantle --bin mantle cargo_free_self_build::tests::receipt_bound_path_aliases_ -- --nocapture
```

```text
running 6 tests
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_member_name_conflict ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_unsafe_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_linker_for_collect2 ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_target_prefixed_member_name ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 782 filtered out; finished in 0.00s
```
