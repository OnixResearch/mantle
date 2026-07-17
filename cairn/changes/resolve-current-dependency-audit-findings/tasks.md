## Implementation

- [x] [serial] I1 Capture and classify the current advisory, source, and license failures plus the exact affected dependency paths. r[verification_evidence.dependency_audit_actionable_findings]
  - Evidence: `evidence/baseline.md` records the checked-policy failures, targeted update dry run, dependency paths, classifications, success contract, false-completion cases, and non-claims.
- [ ] [serial] I2 Generate the minimal compatible `crossbeam-epoch` lock update and prove the vulnerable version is absent without adding a waiver. r[verification_evidence.dependency_audit_actionable_findings]
- [ ] [serial] I3 Add only the exact reviewed Git repository and compound SPDX expression to policy while retaining fail-closed defaults. r[verification_evidence.dependency_audit_actionable_findings]
- [ ] [serial] I4 Refresh operator documentation and paired positive/negative evidence with explicit security, source-authority, license, and non-claim boundaries. r[verification_evidence.dependency_audit_actionable_findings]

## Verification

- [ ] [serial] V1 Run `SNIX_BUILD_SANDBOX_SHELL=/bin/sh nix develop -c cargo-deny check --config deny.toml` and record all four audit classes plus license and source results. r[verification_evidence.dependency_audit_actionable_findings]
- [ ] [serial] V2 Run locked focused compile/tests for the consumers of the moved dependency and the ordinary first-party quality gate. r[verification_evidence.dependency_audit_actionable_findings]
- [ ] [serial] V3 Run immutable Nickel export pin checks and negative missing/default-policy evidence validation. r[verification_evidence.dependency_audit_actionable_findings]
- [ ] [serial] V4 Run `git diff --check`, Cairn validation, proposal/design/tasks gates, and Tracey coverage; archive only after accepted requirements and evidence agree. r[verification_evidence.dependency_audit_actionable_findings]
