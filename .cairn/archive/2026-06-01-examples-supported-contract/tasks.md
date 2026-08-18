# Tasks

## Catalog and inventory

- [x] [serial] Add `examples/catalog.ncl` with typed entries for every checked-in user-facing example, including support tier, capabilities, network behavior, and validation rail. r[examples.support_catalog] Evidence: `evidence/implementation-validation-2026-06-01.md` records `cargo test -p mantle --test examples_inventory -- --nocapture` (`7 passed`) after adding `examples/catalog.ncl` with typed catalog fields.
- [x] [serial] Add a pure inventory checker/test that reads the catalog and checked-in example paths, then rejects missing paths, duplicate ids, duplicate paths, unsupported tiers, and silent skips. r[examples.support_catalog] Evidence: `tests/examples_inventory.rs`; `evidence/implementation-validation-2026-06-01.md` records positive and negative inventory tests passing (`7 passed`).
- [x] [serial] Classify generated seed-dependent, heavyweight, benchmark, and real-network examples explicitly so fast validation knows why each is included or skipped. r[examples.support_catalog] Evidence: `examples/catalog.ncl`; `evidence/implementation-validation-2026-06-01.md` records the catalog inventory test that rejects silent skips and verifies lane inventory.

## Documentation drift

- [x] [serial] Update `examples/README.md` and the root README examples section from the catalog inventory, preserving exact commands and capability notes. r[examples.documentation_drift] Evidence: `examples/README.md`, `README.md`, and `evidence/implementation-validation-2026-06-01.md` (`examples_catalog_covers_checked_in_user_facing_examples ... ok`).
- [x] [serial] Add positive and negative README drift tests for missing catalog entries, stale links, omitted supported examples, invalid command snippets, and stale Crunch branding outside exact compatibility identifiers. r[examples.documentation_drift] Evidence: `tests/examples_inventory.rs`; `evidence/implementation-validation-2026-06-01.md` records negative tests for duplicate paths, invalid tiers, missing README entries, stale refs, and stale Crunch branding.

## Verification

- [x] [serial] Run the examples inventory tests and record the command output as durable evidence before marking this change complete. r[examples.support_catalog] r[examples.documentation_drift] Evidence: `evidence/implementation-validation-2026-06-01.md` records `cargo test -p mantle --test examples_inventory -- --nocapture` with `test result: ok. 7 passed; 0 failed`.
- [x] [serial] Run `cairn validate --root .` and the tasks gate, then archive only after completed tasks cite durable evidence. r[examples.support_catalog] r[examples.documentation_drift] Evidence: `evidence/implementation-validation-2026-06-01.md` records `cairn validate --root .` with `"valid": true` and `cairn gate tasks examples-supported-contract --root .` with `"verdict": "PASS"`.
