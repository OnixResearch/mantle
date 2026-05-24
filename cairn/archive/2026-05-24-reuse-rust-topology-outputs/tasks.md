# Tasks: reuse-rust-topology-outputs

## Implementation

- [x] [serial] r[rust_package_planning.unit_execution.topology.output_reuse] Persist per-unit execution receipts beside declared Rust topology outputs.
- [x] [serial] r[rust_package_planning.unit_execution.topology.output_reuse] Reuse prior outputs only when explicit current inputs, toolchain identity, args digest, and output BLAKE3 digests match the prior receipt.
- [x] [serial] r[rust_package_planning.unit_execution.topology.output_reuse_blockers] Fail closed before rustc when prior cached output evidence is stale, malformed, or missing required artifacts.

## Verification

- [x] [serial] r[rust_package_planning.unit_execution.topology.output_reuse] Add a positive CLI fixture proving repeated `--execute-topology` emits reused unit receipts with stable output digests.
- [x] [serial] r[rust_package_planning.unit_execution.topology.output_reuse_blockers] Add a negative CLI fixture proving a removed cached output artifact returns a structured stale-cache blocker.
- [x] [serial] r[rust_package_planning.unit_execution.topology.output_reuse] Run focused Rust-plan/CLI tests plus Cairn gates.
