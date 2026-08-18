# Tasks

## Spec

- [x] [serial] Add host duplicate identity requirement and design. r[rust_package_planning.native_host_real_unit_identity]

## Implementation

- [x] [serial] Replace single-value host unit key map with identity-preserving host unit selection. r[rust_package_planning.native_host_real_unit_identity]
- [x] [serial] Bind host artifacts from exact selected Cargo edges when available. r[rust_package_planning.native_host_real_unit_identity]
- [x] [serial] Add fail-closed ambiguity blockers for host fallback lookup. r[rust_package_planning.native_host_real_unit_identity]
- [x] [serial] Add duplicate proc-macro and custom-build host unit regression tests. r[rust_package_planning.native_host_real_unit_identity]

## Verification

- [x] [serial] Run focused host identity tests, unit-variant tests, native host graph tests, clean self-probe, and Cairn validation. r[rust_package_planning.native_host_real_unit_identity]
  - Evidence: `cairn/archive/2026-05-29-rust-topology-host-real-unit-identity/evidence/verification.md` records focused tests plus dirty task 292 and clean task 294 (`topology_execution_status=success`, `executions=610`).
