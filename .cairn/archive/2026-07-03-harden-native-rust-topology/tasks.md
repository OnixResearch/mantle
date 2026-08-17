## Implementation

- [x] [serial] I1 Extend native topology diagnostics to include stable unit identity, package identity, role, selected triple, target kind, artifact role, and blocker class where available. r[rust_package_planning.native_topology_hardening]
  Evidence: `cairn/changes/harden-native-rust-topology/evidence.md` records `RustUnitDiagnosticContext` and focused tests.
- [x] [serial] I2 Define a bounded replay receipt shape for failing or blocked native Rust units, using BLAKE3 digests for large inputs. r[rust_package_planning.native_topology_hardening]
  Evidence: `cairn/changes/harden-native-rust-topology/evidence.md` records `RustUnitReplayEvidence` and malformed/oversized input tests.
- [x] [serial] I3 Add edge-case fixtures for host build-script dependencies, proc-macro dependencies, selected target features, and source-root host/target splits. r[rust_package_planning.native_topology_hardening]
  Evidence: `cairn/changes/harden-native-rust-topology/evidence.md` records the focused rust-plan suite covering those existing and added fixtures.
- [x] [serial] I4 Keep added planner logic in pure functions with thin execution-shell plumbing. r[rust_package_planning.native_topology_hardening]
  Evidence: replay/diagnostic construction and validation are pure helpers; execution shells only attach the computed fields to receipts.

## Verification

- [x] [serial] V1 Positive: run focused native topology fixtures that exercise host/target split, proc-macro, and build-script metadata cases. r[rust_package_planning.native_topology_hardening]
  Evidence: `cargo test -p mantle --bin mantle rust_plan::tests:: -- --test-threads=1 --nocapture` passed; see `evidence.md`.
- [x] [serial] V2 Negative: assert wrong-role, wrong-triple, wrong-source-package, and wrong-metadata artifacts fail before rustc with deterministic diagnostics. r[rust_package_planning.native_topology_hardening]
  Evidence: existing `role_validation_rejects_*` tests plus the focused suite pass in `evidence.md`.
- [x] [serial] V3 Negative: assert malformed or oversized replay receipt inputs fail closed without hidden host-tool fallback. r[rust_package_planning.native_topology_hardening]
  Evidence: added `replay_evidence_json_rejects_malformed_and_oversized_inputs` and `replay_evidence_json_rejects_invalid_digest_fields`; see `evidence.md`.
- [x] [serial] V4 Run focused Rust topology tests, `cargo fmt -p mantle --check`, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[rust_package_planning.native_topology_hardening]
  Evidence: focused tests, fmt, diff check, Cairn validation, and proposal/design/tasks gates are recorded in `evidence.md`.
