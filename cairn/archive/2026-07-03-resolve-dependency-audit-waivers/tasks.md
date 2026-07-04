## Implementation

- [x] [serial] I1 Re-audit remaining dependency waivers and record exact transitive paths, advisory IDs, and current upstream constraints. r[verification_evidence.dependency_audit_waiver_resolution]
  - Evidence: `cairn/changes/resolve-dependency-audit-waivers/evidence/validation.md` records checked-in policy audit output, cargo-tree paths, temporary waiver-removal probes, and upgrade probes.
- [x] [serial] I2 Attempt minimal safe dependency/version/feature changes that retire waived advisories without broad unrelated churn. r[verification_evidence.dependency_audit_waiver_resolution]
  - Evidence: pueue task 122 disabled `postcard` default features in vendored snix crates and pruned only `atomic-polyfill`, `critical-section`, `hash32 0.2.1`, `heapless 0.7.17`, and `spin 0.9.8` from `Cargo.lock`.
- [x] [serial] I3 Update `deny.toml` and dependency-audit docs to remove retired waivers or sharpen upstream-blocked waiver reasons and unblock conditions. r[verification_evidence.dependency_audit_upstream_blockers]
  - Evidence: `RUSTSEC-2023-0089` was removed; retained `paste`, `proc-macro-error2`, and `quick-xml` waivers now name exact paths and unblock conditions in `deny.toml` and `docs/dependency-audit.md`.
- [x] [serial] I4 Add or update a guard so dependency audit evidence must use the checked-in policy. r[verification_evidence.dependency_audit_policy_regression]
  - Evidence: `scripts/check-dependency-audit-evidence.rs` validates evidence transcripts, and pueue tasks 119/179 prove positive and negative guard behavior.

## Verification

- [x] [serial] V1 Run `cargo-deny check --config deny.toml` and capture advisory/license/source results with the checked-in policy. r[verification_evidence.dependency_audit_policy_regression]
  - Evidence: pueue task 134 reports `advisories ok, bans ok, licenses ok, sources ok` with the updated checked-in policy and no `RUSTSEC-2023-0089` / `atomic-polyfill` matches.
- [x] [serial] V2 Run a negative missing/default-policy audit guard and prove it is not accepted as evidence. r[verification_evidence.dependency_audit_policy_regression]
  - Evidence: pueue task 179 validates the change evidence and rejects a raw transcript with `negative_raw_transcript_status=1` for missing command and `--config deny.toml` policy evidence.
- [x] [serial] V3 Run focused compile checks for crates affected by dependency movement. r[verification_evidence.dependency_audit_waiver_resolution]
  - Evidence: pueue task 137 runs `cargo check -p snix-build -p snix-castore -p snix-store --locked` successfully after the `postcard` feature change.
- [x] [serial] V4 Run `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.dependency_audit_upstream_blockers]
  - Evidence: pueue task 189 passes formatting, guard, evidence, whitespace, and Cairn validation/gates; final post-task-edit gates will be rerun before archive.
