# Tasks

## Spec

- [x] [serial] Add native executor hardening requirement and design. r[rust_package_planning.native_executor_hardening]

## Implementation

- [x] [serial] Extend receipt cache checks to all declared input/artifact/env/toolchain facts. r[rust_package_planning.native_executor_hardening]
  - Evidence: added `environment_digest_blake3` to `RustUnitExecutionReceipt`, reuse matching, stale-cache blockers, and focused env-change negative test. Verified by pueue task 414: `cargo test -p mantle --bin mantle rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph`, `cargo test -p mantle --test rust_plan_cli rust_plan_cli_reuses_unified_topology_outputs_on_repeat_run`, and `cargo test -p mantle --test rust_plan_cli rust_plan_cli_blocks_stale_unified_topology_cached_output` all passed.
- [x] [serial] Validate dependency and host artifact files and digests before rustc. r[rust_package_planning.native_executor_hardening]
  - Evidence: existing `artifact_digests(...)` preflight blocks missing dependency/host artifacts before `rustc`, and successful dependency-chain receipts bind produced `.rlib` digests. Verified by pueue task 466: missing dependency artifact, missing host artifact, and produced dependency-chain tests all passed.
- [x] [serial] Normalize deterministic blocker classes for stale, missing, and mismatched material. r[rust_package_planning.native_executor_hardening]
  - Evidence: missing source, source-closure, missing declared-output, and stale/mismatched cache cases emit deterministic blocker classes before dependent execution. Verified by pueue task 467; task 414 additionally covers env-mismatch stale cache.
- [x] [serial] Redact and stabilize rustc diagnostics in failure receipts. r[rust_package_planning.native_executor_hardening]
  - Evidence: `redacted_diagnostic(...)` now uses named line bounds, strips NULs, redacts `/tmp`/`/var/tmp` paths, and preserves non-temp source context. Verified by pueue task 468: redaction positive/negative tests and supported-unit execution test passed.
- [x] [serial] Add positive cache-hit and negative stale-output tests. r[rust_package_planning.native_executor_hardening]
  - Evidence: `executes_first_supported_lib_unit_from_derivation_graph` now asserts positive output reuse and negative env-mismatch stale-cache blocking; CLI stale-output coverage remains in `rust_plan_cli_blocks_stale_unified_topology_cached_output`. Verified by pueue tasks 414 and 467.

## Verification

- [ ] [serial] Run executor cache/rebuild tests, receipt replay tests, topology self-probe, and Cairn validation. r[rust_package_planning.native_executor_hardening]
