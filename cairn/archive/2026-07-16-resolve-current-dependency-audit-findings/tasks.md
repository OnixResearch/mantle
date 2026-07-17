## Implementation

- [x] [serial] I1 Capture and classify the current advisory, source, and license failures plus the exact affected dependency paths. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `evidence/baseline.md` records the checked-policy failures, targeted update dry run, dependency paths, classifications, success contract, false-completion cases, and non-claims.
- [x] [serial] I2 Generate the minimal compatible `crossbeam-epoch` lock update and prove the vulnerable version is absent without adding a waiver. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `cargo update -p crossbeam-epoch --precise 0.9.20` changed only that package checksum/version in `Cargo.lock`; locked metadata now resolves `0.9.20`, the checked-policy audit reports advisories `ok`, and no `RUSTSEC-2026-0204` waiver exists.
- [x] [serial] I3 Add only the exact reviewed Git repository and compound SPDX expression to policy while retaining fail-closed defaults. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `deny.toml` retains both unknown-source denials and license confidence while admitting only the reviewed repository and exact expression; temporary policies omitting each new admission failed on the expected source/license class.
- [x] [serial] I4 Refresh operator documentation and paired positive/negative evidence with explicit security, source-authority, license, and non-claim boundaries. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `docs/dependency-audit.md` records the fixes and claim boundary; `evidence/validation.md` records the approach registry, positive/negative checks, offline-vendor boundary, and residual non-claims.

## Verification

- [x] [serial] V1 Run `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml` and record all four audit classes plus license and source results. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `evidence/validation.md` records pueue task `70`: `advisories ok, bans ok, licenses ok, sources ok`.
- [x] [serial] V2 Run locked focused compile/tests for the consumers of the moved dependency and the ordinary first-party quality gate. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `evidence/validation.md` records pueue task `47` locked checks, including 219 passing `crunch-store` tests, and task `55` passing the full first-party quality gate.
- [x] [serial] V3 Run immutable Nickel export pin checks and negative missing/default-policy evidence validation. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `evidence/validation.md` records task `48` pin/evidence self-tests and task `50` rejection of policies missing either exact new admission.
- [x] [serial] V4 Run `git diff --check`, Cairn validation, proposal/design/tasks gates, and Tracey coverage; archive only after accepted requirements and evidence agree. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `evidence/validation.md` records task `79` passing whitespace, evidence, validation, and all three gates; task `81` records the unrelated release-provenance Tracey profile at `140/145` with the exact five missing IDs and no dangling references.
