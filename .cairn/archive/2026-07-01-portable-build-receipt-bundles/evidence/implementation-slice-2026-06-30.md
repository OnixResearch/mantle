# Implementation slice evidence — Portable build receipt bundles

Date: 2026-06-30

## Implemented in this slice

- Added `src/portable_receipt.rs` with a Mantle-owned `mantle-build-receipt-bundle-v1` evidence bundle model.
- Added deterministic record ordering, BLAKE3 bundle digesting, trust snapshot binding fields, policy-hash binding, named limits, duplicate/conflict rejection, strong-claim evidence completeness classification, non-claim wording, and idempotent import into Mantle evidence state.
- Added `mantle receipt bundle export|list|verify|import` CLI wiring with JSON and human reports.

## Current focused evidence

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-store-archive cargo test -p mantle --bin mantle portable_receipt::tests
running 5 tests
...
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 948 filtered out
```

## Remaining gap before archive

This is not yet a complete drain of `portable-build-receipt-bundles`: export/list does not yet gather live action receipts, source refs, PathInfo identities, attestations, sandbox reports, or semantic graph edges from existing state; verify/import is not paired with store archives or source bundles; `mantle why` semantic graph integration and CLI conflict-diagnostic tests still need to land.
