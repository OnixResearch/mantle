# Tasks

## Spec

- [x] [serial] Add native executor hardening requirement and design. r[rust_package_planning.native_executor_hardening]

## Implementation

- [x] [serial] Extend receipt cache checks to all declared input/artifact/env/toolchain facts. r[rust_package_planning.native_executor_hardening]
  - Evidence: added `environment_digest_blake3` to `RustUnitExecutionReceipt`, reuse matching, stale-cache blockers, and focused env-change negative test. Verified by pueue task 414: `cargo test -p mantle --bin mantle rust_plan::tests::executes_first_supported_lib_unit_from_derivation_graph`, `cargo test -p mantle --test rust_plan_cli rust_plan_cli_reuses_unified_topology_outputs_on_repeat_run`, and `cargo test -p mantle --test rust_plan_cli rust_plan_cli_blocks_stale_unified_topology_cached_output` all passed.
- [ ] [serial] Validate dependency and host artifact files and digests before rustc. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Normalize deterministic blocker classes for stale, missing, and mismatched material. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Redact and stabilize rustc diagnostics in failure receipts. r[rust_package_planning.native_executor_hardening]
- [ ] [serial] Add positive cache-hit and negative stale-output tests. r[rust_package_planning.native_executor_hardening]

## Verification

- [ ] [serial] Run executor cache/rebuild tests, receipt replay tests, topology self-probe, and Cairn validation. r[rust_package_planning.native_executor_hardening]
