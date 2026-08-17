# Provider-backed closure status focused validation

Task-ID: V1
Covers: rust_package_planning.source_built_toolchain_closure.provider_status

## Result

Passed.

## Commands

Focused pueue task 13:

```text
rustfmt src/source_toolchain_closure.rs src/cargo_free_self_build.rs
rustfmt --check src/source_toolchain_closure.rs src/cargo_free_self_build.rs
git diff --check
cargo test -p mantle --bin mantle source_toolchain_closure::tests::provided_rust_provider_status_promotes_source_built_claim -- --nocapture
cargo test -p mantle --bin mantle cargo_free_self_build::tests::fixed_point_summary_omits_closure_non_claim_when_provider_supplies_claim -- --nocapture
cargo test -p mantle --bin mantle cargo_free_self_build::tests::effective_closure_ -- --nocapture
cargo test -p mantle --bin mantle cargo_free_self_build::tests::rust_source_provider_binding_ -- --nocapture
```

Visible result excerpt:

```text
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 781 filtered out; finished in 0.00s
...
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 782 filtered out; finished in 0.00s
```

Final module pueue task 22:

```text
rustfmt src/source_toolchain_closure.rs src/cargo_free_self_build.rs
rustfmt --check src/source_toolchain_closure.rs src/cargo_free_self_build.rs src/rust_plan.rs src/rust_source_provider.rs
git diff --check
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
cargo test -p mantle --bin mantle cargo_free_self_build -- --nocapture --test-threads=1
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Visible result excerpt:

```text
test cargo_free_self_build::tests::self_build_non_claims_omit_closure_non_claim_when_provider_supplies_claim ... ok
...
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 754 filtered out; finished in 0.02s

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
