# Rust source bootstrap route anchor

Task-ID: rust-source-bootstrap-route-2026-06-10
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now has a typed route artifact at `bootstrap/rust-source-plan.ncl` for the next real Rust-from-source provider step. The plan records an mrustc-based source ladder through Rust `1.90.0`, `1.91.1`, `1.92.0`, `1.93.1`, and final `1.94.0`, plus the final provider output roles required by `mantle-rust-source-provider-v1`.

This is not provider evidence. `bootstrap/rust-source.ncl` still fails closed, and no `rustc`/`cargo`/rustlib provider output or fixed-point proof has been produced by this change.

## Reference evidence inspected

- `/home/brittonr/git/reference-repos/stagex--stagex/packages/core/rust/package.toml` lines 8-32 list `mrustc` `0.12.0` and Rust source tarballs `1.90.0` through `1.94.0` with SHA-256 digests.
- `/home/brittonr/git/reference-repos/stagex--stagex/packages/core/rust/Containerfile` lines 144-190 build `rustc`/`cargo` with `mrustc`/`minicargo` and materialize the bootstrap prefix.
- `/home/brittonr/git/reference-repos/stagex--stagex/packages/core/rust/Containerfile` lines 337-356 show the successive `1.91.1` → `1.94.0` source build route.
- `/home/brittonr/git/nixpkgs/pkgs/development/compilers/mrustc/bootstrap.nix` lines 20-29 and 83-136 show the older Nixpkgs mrustc bootstrap shape, but the checked-in Mantle plan follows the Stagex current-version route instead.

## Commands and output

### source toolchain closure route tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

Exit status: `0`

```text
test source_toolchain_closure::tests::validator_allows_seed_reason_with_marker_letters_inside_larger_word ... ok
test source_toolchain_closure::tests::valid_manifest_yields_stable_order_independent_policy_digest ... ok
test source_toolchain_closure::tests::validator_rejects_seed_member_without_seed_exception ... ok
test source_toolchain_closure::tests::validator_rejects_missing_required_role ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test source_toolchain_closure::tests::validator_rejects_placeholder_seed_exception_reason ... ok
test source_toolchain_closure::tests::seed_exception_policy_digest_is_order_independent_and_accounted ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_member_without_receipt ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_rustc_with_prebuilt_source_marker ... ok
test source_toolchain_closure::tests::validator_rejects_unverified_seed_exception_name ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_bootstrap_plan_yields_stable_digest ... ok
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 716 filtered out; finished in 0.04s
```

### provider shell regression tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture
```

Exit status: `0`

```text
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test rust_source_provider::tests::import_provider_rejects_prebuilt_metadata_without_output ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test tests::bootstrap_rust_source_provider_action_parses_smoke_evidence_dir ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok

test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.03s
```

## Lifecycle validation

### cairn validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Exit status: `0`

```text
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

### cairn tasks gate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Exit status: `0`

```text
{
  "change": "source-built-rust-seed-closure",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "4d1ec7f6e90516ca3dbd108892d219cd1e2bdc4a54e5dff384e0f70ea749beb8",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "ec47977164c5b88ab20d52bc08c574d8e4c67416e89a14096249dc2e6e58cd03",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The route is only a typed implementation anchor. The unchecked provider materialization, real provider smoke, and fixed-point proof tasks remain blocked until Mantle can build the route into a receipt-bound provider directory containing real `bin/rustc`, `bin/cargo`, host rustlib, target rustlib, provider metadata, and receipts.
