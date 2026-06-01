# Toolchain leakage enforcement validation

Task-ID: I2
Covers: rust_package_planning.source_built_toolchain_closure
Date: 2026-05-31
Owner: agent

## Implemented slice

- `src/source_toolchain_closure.rs` now has a pure observed-input enforcement core: declared receipt-bound toolchain members must match observed role, execution path, and file BLAKE3 digest when a digest is observed.
- `src/cargo_free_self_build.rs` now enforces source-built closure mode before Cargo-free Rust unit execution:
  - resolves the selected `rustc` and rejects undeclared or digest-mismatched rustc paths,
  - checks `rustc --print sysroot` against the declared sysroot path,
  - checks declared linker, C compiler, pkg-config, and native-helper executable digests before exposing them,
  - replaces ambient child `PATH` with a receipt-bound guard directory of declared tool aliases plus the existing forbidden Cargo shim,
  - keeps undeclared Nix profile tools out of the receipt-bound PATH alias set,
  - keeps `claim=false` and `not-source-built-toolchain-closure` while reporting status `validated-enforced` for matching manifests.
- `tests/cargo_free_self_build_cli.rs` proves matching real-host toolchain manifests are accepted without source-built proof claim, and mismatched rustc manifests fail before unit execution.
- Review-fix tests in `src/cargo_free_self_build.rs` now explicitly cover linker digest mismatch, pkg-config digest mismatch, and undeclared Nix profile tools.

This does not complete the real source-root/seed provider or end-to-end source-built toolchain closure proof tasks.

## Baseline before edits

Pueue task `49` ran the focused baseline command:

```text
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
cargo test -p mantle --bin mantle cargo_free -- --nocapture
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture
```

Result:

```text
Task 49 Success
source_toolchain_closure: 11 passed
cargo_free: 16 passed
cargo_free_self_build_cli: 11 passed
```

## Post-change validation

Pueue task `60` ran the same focused command with the documented Mantle test environment:

```text
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/home/brittonr/.cargo/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
```

Output excerpts:

```text
running 15 tests
...
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 596 filtered out; finished in 0.00s

running 23 tests
...
test cargo_free_self_build::tests::receipt_bound_path_aliases_expose_declared_pkg_config_only ... ok
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok
test cargo_free_self_build::tests::receipt_bound_path_env_omits_ambient_path_entries ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_pkg_config_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_sysroot_leakage ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_c_compiler_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_linker_digest_mismatch ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 591 filtered out; finished in 0.00s

running 12 tests
...
test cargo_free_self_build_rejects_undeclared_host_rustc_before_unit_execution ... ok
test cargo_free_self_build_enforces_matching_toolchain_closure_manifest_without_claiming_proof ... ok
test cargo_free_fixed_point_enforces_matching_toolchain_closure_manifest_without_claiming_proof ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
```

Review-fix focused command after adding explicit linker/pkg-config/Nix-profile coverage:

```text
cargo test -p mantle --bin mantle cargo_free -- --nocapture
cargo test -p mantle --test cargo_free_self_build_cli -- --nocapture
```

Pueue task `62` output excerpts:

```text
running 23 tests
...
test cargo_free_self_build::tests::receipt_bound_path_aliases_reject_undeclared_nix_profile_tools ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_pkg_config_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_c_compiler_digest_mismatch ... ok
test cargo_free_self_build::tests::receipt_bound_enforcement_rejects_linker_digest_mismatch ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 591 filtered out; finished in 0.00s

running 12 tests
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s
```

Formatter and Cairn checks after the evidence update candidate:

```text
cargo fmt --check -p mantle -v
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-task-update Cairn check

After marking I2 complete in `tasks.md`, this command was run:

```text
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "c450105f2a71800b5ea7a280486d2273a524ae0d9ecfabb1062a0b96a2745787",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ec76ccce270e34e151652d696bb8ac72e86d6211e8c2545e6d96404ff4926883",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-review-fix Cairn check

After adding explicit linker, pkg-config, and Nix-profile coverage, this command was run:

```text
cargo fmt --check -p mantle -v
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 4,
  "valid": true
}
{
  "change": "source-built-toolchain-closure",
  "input_hash": "10c238e15ce4b0ac6872e399a39b9df569c085eb7707d76098cfe3feb19abae7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "713874ea73ff2b9f7e8cc11355399c2ee683faedf047453e1d3b28a3c15e50de",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Decision

I2 can be checked for the bounded implementation slice: Cargo-free self-build/fixed-point planning accepts receipt-bound closure manifests and blocks each named leakage class before Rust unit execution. The status still refuses a source-built closure claim until the real source-root/seed provider and end-to-end proof tasks complete.

## Next action

Thread the closure policy digest into per-stage receipts and require stage1/stage2 policy digest equality before any future source-built closure success report.
