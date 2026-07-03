## Implementation

- [ ] [serial] I1 Re-audit remaining dependency waivers and record exact transitive paths, advisory IDs, and current upstream constraints. r[verification_evidence.dependency_audit_waiver_resolution]
- [ ] [serial] I2 Attempt minimal safe dependency/version/feature changes that retire waived advisories without broad unrelated churn. r[verification_evidence.dependency_audit_waiver_resolution]
- [ ] [serial] I3 Update `deny.toml` and dependency-audit docs to remove retired waivers or sharpen upstream-blocked waiver reasons and unblock conditions. r[verification_evidence.dependency_audit_upstream_blockers]
- [ ] [serial] I4 Add or update a guard so dependency audit evidence must use the checked-in policy. r[verification_evidence.dependency_audit_policy_regression]

## Verification

- [ ] [serial] V1 Run `cargo-deny check --config deny.toml` and capture advisory/license/source results with the checked-in policy. r[verification_evidence.dependency_audit_policy_regression]
- [ ] [serial] V2 Run a negative missing/default-policy audit guard and prove it is not accepted as evidence. r[verification_evidence.dependency_audit_policy_regression]
- [ ] [serial] V3 Run focused compile checks for crates affected by dependency movement. r[verification_evidence.dependency_audit_waiver_resolution]
- [ ] [serial] V4 Run `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[verification_evidence.dependency_audit_upstream_blockers]
