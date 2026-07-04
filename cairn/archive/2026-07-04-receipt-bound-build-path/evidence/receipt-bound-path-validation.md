# Receipt-bound PATH validation

Date: 2026-07-04

## Summary

Implemented strict receipt-bound search-path planning for build environments:

- Pure search-path model classifies entries as declared store tool refs, alias views, or host-inventory records and computes a BLAKE3 digest over the ordered plan.
- Strict build-request normalization accepts declared store-tool `PATH` entries, records tool refs in `BuildEnvironmentReport.search_path`, and rejects ambient/unclassified `PATH` entries before execution.
- JSON build reports now expose search-path digest, entries, aliases, and real tool refs; `docs/build-correctness-primitives.md` documents the operator-facing evidence.

## Baseline

Before the change, the focused build-request baseline passed:

```text
Command: cargo test -p crunch-build build_request:: -- --nocapture
Result: 31 passed; 0 failed; 0 ignored; 0 measured; 436 filtered out
Evidence source: pueue task 517 output from this drain session.
```

## Validation transcript

Captured in `/tmp/mantle-receipt-path-tests.txt` from pueue task 72.

```text
## cargo fmt check
cargo fmt -p crunch-build -p crunch-pipeline -p mantle -- --check

## cargo test -p crunch-build build_request::
...
running 33 tests
...
test build_request::tests::strict_mode_rejects_ambient_path_poisoning_with_receipt_bound_report ... ok
...
test build_request::tests::strict_mode_accepts_declared_store_path_and_reports_tool_refs ... ok
test build_request::tests::strict_mode_reports_stable_environment_digest ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 434 filtered out; finished in 0.00s

## cargo test -p crunch-build environment_policy::
...
running 5 tests
...
test environment_policy::tests::receipt_bound_search_path_plan_rejects_ambient_entries_and_alias_drift ... ok
...
test environment_policy::tests::receipt_bound_search_path_plan_records_ordered_digest_aliases_and_real_refs ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 462 filtered out; finished in 0.00s

## cargo test -p mantle --bin crunch build_report::
...
running 10 tests
...
test build_report::tests::render_build_json_report_serializes_full_substitution_fields_stably ... ok
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 1175 filtered out; finished in 0.03s
```

## Cairn gates

Captured in `/tmp/mantle-receipt-path-cairn.txt` from pueue task 97.

```text
## cairn validate
{
  "changes": 2,
  "specs_validated": 18,
  "valid": true
}

## cairn gate proposal
{
  "change": "receipt-bound-build-path",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

## cairn gate design
{
  "change": "receipt-bound-build-path",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

## cairn gate tasks
{
  "change": "receipt-bound-build-path",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Post-archive validation

Captured in `/tmp/mantle-receipt-path-archive.txt` from pueue task 104 after sync and archive.

```text
## cairn sync receipt-bound-build-path --execute
{
  "blocked": false,
  "change": "receipt-bound-build-path",
  "dry_run": false,
  "mutated": true
}

## cairn archive receipt-bound-build-path --execute
{
  "blocked": false,
  "change": "receipt-bound-build-path",
  "dry_run": false,
  "mutated": true
}

## cairn validate after archive
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "spec_issues": [],
  "specs_validated": 17,
  "valid": true
}
```
