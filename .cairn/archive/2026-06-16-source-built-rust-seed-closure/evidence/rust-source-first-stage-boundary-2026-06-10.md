# Rust source first-stage materializer boundary

Task-ID: rust-source-first-stage-boundary-2026-06-10
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now turns the typed route in `bootstrap/rust-source-plan.ncl` into a first executable materializer boundary for the mrustc seed stage. `mantle bootstrap rust-source-provider` loads and validates the route plan, selects the single `mrustc-seed` stage, derives the expected first-stage output layout from the provider roles, and writes two scratch artifacts before failing closed:

- `mrustc-first-stage-plan.json` — route digest, policy digest, selected stage, sources, expected output roles/paths, and planned directories.
- `run-mrustc-first-stage.sh` — executable boundary script that checks for declared first-stage sources and exits before provider output creation when sources/build implementation are absent.

This does not produce `bin/rustc`, `bin/cargo`, rustlib outputs, provider metadata, or receipts. It preserves the deterministic source-built Rust provider blocker and still creates no provider output directory.

## Baseline before changes

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
test cargo_free_self_build::tests::rust_source_provider_binding_uses_validated_provider_rustc ... ok
test rust_source_provider::tests::smoke_provider_rejects_invalid_metadata_before_launch ... ok
test tests::bootstrap_rust_source_provider_rejects_smoke_evidence_without_smoke ... ok
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::bootstrap_rust_source_provider_action_parses_smoke_evidence_dir ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test tests::bootstrap_rust_source_provider_action_parses_import_dir ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok

test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.09s
```

## Validation after changes

### provider boundary tests

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
test tests::self_build_cli_rejects_rust_source_provider_without_cargo_free ... ok
test tests::self_build_cli_accepts_cargo_free_rust_source_provider ... ok
test tests::bootstrap_rust_source_provider_action_parses ... ok
test rust_source_provider::tests::smoke_provider_runs_synthetic_rustc_after_validation ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok
test tests::bootstrap_rust_source_provider_fails_closed_without_output ... ok
mantle rust source first stage: mrustc-to-rust-1.90.0
missing source 'mrustc-0.12.0' from https://example.invalid/mrustc-0.12.0.tar.gz with sha256 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
test rust_source_provider::tests::first_stage_script_fails_before_provider_output_when_sources_missing ... ok
test rust_source_provider::tests::import_provider_copies_validated_provider ... ok
test rust_source_provider::tests::smoke_evidence_rejects_tampered_smoke_output ... ok
test rust_source_provider::tests::smoke_evidence_persists_summary_output_and_metadata ... ok

test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.08s
```

### route/core regression tests

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
test source_toolchain_closure::tests::validated_closure_status_keeps_claim_disabled_until_enforcement_lands ... ok
test source_toolchain_closure::tests::validator_accepts_explicit_seed_exception ... ok
test source_toolchain_closure::tests::validator_allows_seed_reason_with_marker_letters_inside_larger_word ... ok
test source_toolchain_closure::tests::validator_rejects_missing_required_role ... ok
test source_toolchain_closure::tests::validator_rejects_placeholder_seed_exception_reason ... ok
test source_toolchain_closure::tests::validator_rejects_seed_member_without_seed_exception ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_metadata_yields_stable_digest ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_member_without_receipt ... ok
test source_toolchain_closure::tests::validator_rejects_source_built_rustc_with_prebuilt_source_marker ... ok
test source_toolchain_closure::tests::validator_rejects_unverified_seed_exception_name ... ok
test source_toolchain_closure::tests::valid_rust_source_provider_bootstrap_plan_yields_stable_digest ... ok
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 718 filtered out; finished in 0.04s
```

## Additional checks

### touched-provider rustfmt check

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/run/current-system/sw/bin:/usr/bin:/bin rustfmt --check src/rust_source_provider.rs
```

Exit status: `0`

```text
completed successfully
```

Note: the broader `cargo fmt --check -p mantle -v` and `rustfmt --check src/main.rs src/rust_source_provider.rs` checks currently print an unrelated pre-existing diff in `src/build_report.rs`; this slice intentionally did not churn that file.

### diff whitespace check

Command:

```sh
git diff --check
```

Exit status: `0`

```text
completed successfully
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
  "input_hash": "b2cbed0b8902a4e64768ecb43018734f894fe9532f6464446d27a8b0c3f6e1ec",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "86a55feb6178160d87a185ecbb2a224bd9e79141fa40819811905541913e7e10",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The first-stage boundary is executable but still intentionally fails closed. The provider task remains unchecked until the boundary actually fetches/materializes the declared sources, runs the mrustc stage, emits real provider artifacts, writes receipt-bound metadata, and passes provider smoke/fixed-point proofs.
