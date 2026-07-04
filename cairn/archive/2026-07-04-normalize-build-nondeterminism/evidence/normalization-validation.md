# Determinism normalization validation

Date: 2026-07-04

## Summary

Implemented strict determinism normalization evidence for proof-grade builds:

- Build environment reports now bind a BLAKE3 determinism policy digest covering time, timezone, locale, temp roots, host/user metadata, executor umask, modeled randomness, and order-sensitive output processing.
- Strict build-request normalization derives the report from the actual normalized sandbox environment, so declared `SOURCE_DATE_EPOCH` changes alter the policy digest while ambient parent environment remains outside the build request.
- Deterministic release proof receipts now require normalization controls in `normalized_execution_envelope`; missing or unsupported controls block strong claims, and digest drift reports the first modeled divergent output surface.
- Action specs/receipts can bind the determinism policy digest, and JSON build reports expose the normalization report.

## Baseline

Before the change, focused tests passed:

```text
Command: cargo test -p crunch-release-core determinism:: -- --nocapture
Result: 20 passed; 0 failed; 0 ignored; 0 measured; 82 filtered out

Command: cargo test -p crunch-build build_request:: -- --nocapture
Result: 33 passed; 0 failed; 0 ignored; 0 measured; 434 filtered out

Evidence source: `/tmp/mantle-normalization-baseline.txt` from pueue task 158.
```

## Validation transcript

Captured in `/tmp/mantle-normalization-final-tests.txt` from pueue task 110.

```text
## cargo fmt check
cargo fmt -p crunch-build -p crunch-pipeline -p crunch-release-core -p mantle -- --check

## cargo test -p crunch-build environment_policy::
running 7 tests
...
test environment_policy::tests::determinism_normalization_plan_blocks_unsupported_controls_and_divergence ... ok
test environment_policy::tests::determinism_normalization_plan_records_stable_policy_digest ... ok
...
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 463 filtered out; finished in 0.00s

## cargo test -p crunch-build build_request::
running 34 tests
...
test build_request::tests::strict_mode_determinism_policy_changes_with_declared_time_override ... ok
test build_request::tests::strict_mode_reports_stable_environment_digest ... ok
...
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 436 filtered out; finished in 0.00s

## cargo test -p crunch-release-core determinism::
running 21 tests
...
test determinism::tests::deterministic_receipt_accepts_strict_matching_runs ... ok
test determinism::tests::deterministic_receipt_blocks_missing_or_unsupported_normalization_controls ... ok
test determinism::tests::deterministic_receipt_rejects_digest_drift ... ok
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 82 filtered out; finished in 0.00s

## cargo test -p mantle --bin crunch build_report::
running 10 tests
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1176 filtered out; finished in 0.00s

## cargo test -p mantle --bin crunch build_correctness::
running 12 tests
...
test build_correctness::tests::build_correctness_action_spec_binds_determinism_policy_digest ... ok
test build_correctness::tests::build_correctness_action_receipt_is_deterministic_and_bounded ... ok
...
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1174 filtered out; finished in 0.00s
```

## Cairn gates

Captured in `/tmp/mantle-normalization-cairn.txt` from pueue task 117.

```text
## cairn validate
{
  "changes": 1,
  "specs_validated": 17,
  "valid": true
}

## cairn gate proposal
{
  "change": "normalize-build-nondeterminism",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

## cairn gate design
{
  "change": "normalize-build-nondeterminism",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

## cairn gate tasks
{
  "change": "normalize-build-nondeterminism",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-archive validation

Captured in `/tmp/mantle-normalization-archive.txt` from pueue task 119 after sync and archive.

```text
## cairn sync normalize-build-nondeterminism --execute
{
  "blocked": false,
  "change": "normalize-build-nondeterminism",
  "dry_run": false,
  "mutated": true
}

## cairn archive normalize-build-nondeterminism --execute
{
  "blocked": false,
  "change": "normalize-build-nondeterminism",
  "dry_run": false,
  "mutated": true
}

## cairn validate after archive
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}
```

Final validation after archived evidence update (pueue task 13):

```text
{
  "change_issues": [],
  "changes": 0,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 16,
  "valid": true
}
```
