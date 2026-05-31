# Fixture test validation for source-built toolchain closure

Task-ID: fixture-tests
Covers: rust_package_planning.source_built_toolchain_closure

## Scope

This evidence covers positive and negative fixture tests for source-built toolchain closure handling. It does not claim the real end-to-end source-built closure fixed-point proof.

## What changed

- Positive manifest fixture parsing now covers JSON manifest parsing, seed exception accounting, source-built member counts, policy digest length, and order-independent policy digesting across members and seed exceptions.
- Policy digest coverage now proves seed exception justification text affects the digest.
- Negative fixture coverage now rejects placeholder/TODO and unverified seed exception metadata fail-closed before proof promotion.
- Fixed-point policy coverage now rejects stage policy digest presence mismatches and keeps `claim=false` / `not-source-built-toolchain-closure` even when stage binary and policy digests match.
- CLI fixture coverage now accepts matching seed-exception manifests without claiming source-built closure proof and rejects placeholder seed-exception manifests before writing receipts/binaries.

## Test evidence

### pueue task 21 — source_toolchain_closure focused tests

Command:

```sh
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

Result excerpt from `pueue_log 21`:

```text
running 21 tests
test source_toolchain_closure::tests::json_fixture_parses_seed_exception_and_counts_source_built_members ... ok
test source_toolchain_closure::tests::validator_allows_seed_reason_with_marker_letters_inside_larger_word ... ok
test source_toolchain_closure::tests::validator_rejects_placeholder_seed_exception_reason ... ok
test source_toolchain_closure::tests::seed_exception_policy_digest_is_order_independent_and_accounted ... ok
test source_toolchain_closure::tests::policy_digest_changes_when_seed_exception_reason_changes ... ok
test source_toolchain_closure::tests::validator_rejects_unverified_seed_exception_name ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 626 filtered out; finished in 0.00s
```

### pueue task 49 — cargo_free_self_build focused unit tests

Command:

```sh
cargo test -p mantle --bin mantle cargo_free_self_build -- --nocapture
```

Result excerpt from `pueue_log 49`:

```text
running 21 tests
test cargo_free_self_build::tests::fixed_point_status_rejects_policy_digest_presence_mismatch_before_success ... ok
test cargo_free_self_build::tests::fixed_point_summary_keeps_closure_non_claim_even_when_policy_matches ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 625 filtered out; finished in 0.00s
```

### pueue task 50 — existing toolchain-closure CLI fixtures

Command:

```sh
cargo test -p mantle --test cargo_free_self_build_cli toolchain_closure -- --nocapture
```

Result excerpt from `pueue_log 50`:

```text
running 4 tests
test cargo_free_fixed_point_rejects_invalid_toolchain_closure_manifest ... ok
test cargo_free_self_build_rejects_invalid_toolchain_closure_manifest ... ok
test cargo_free_self_build_enforces_matching_toolchain_closure_manifest_without_claiming_proof ... ok
test cargo_free_fixed_point_enforces_matching_toolchain_closure_manifest_without_claiming_proof ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.19s
```

### pueue task 51 — new seed-exception CLI fixtures

Command:

```sh
cargo test -p mantle --test cargo_free_self_build_cli seed_exception -- --nocapture
```

Result excerpt from `pueue_log 51`:

```text
running 2 tests
test cargo_free_self_build_rejects_placeholder_seed_exception_manifest ... ok
test cargo_free_self_build_reports_seed_exception_accounting_without_claiming_proof ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.10s
```

## Final validation

### pueue task 22 — formatting, diff check, Cairn validation, tasks gate

Command:

```sh
cargo fmt --check -p mantle -v
git diff --check
/home/brittonr/.cargo-target/debug/cairn validate --root .
/home/brittonr/.cargo-target/debug/cairn gate tasks source-built-toolchain-closure --root .
```

Result excerpt from `pueue_log 22`:

```text
rustfmt --edition 2024 --check /home/brittonr/git/mantle/examples/benchmark_compare.rs ... /home/brittonr/git/mantle/tests/transcript_cli.rs
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
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```
