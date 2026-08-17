# Tasks

## Contract

- [x] [serial] Define stable project soundness diagnostic classes, severities, subjects, JSON fields, no-network default behavior, and bounded readiness claims. r[operator_diagnostics.project_soundness_diagnostics]
- [x] [serial] Define project workflow soundness checks for manifest, lockfile, generated inputs, patches, mirrors, hash algorithms, freshness, trust, fetch policy, retention roots, and orphaned entries. r[project_workflows.project_soundness_checks]

## Implementation

- [x] [serial] Implement pure soundness classification over parsed manifest, lockfile, generated input facts, and optional probe/trust/retention facts. r[operator_diagnostics.project_soundness_diagnostics] r[project_workflows.project_soundness_checks]
- [x] [serial] Implement CLI rendering for human and JSON output while keeping default check no-network and JSON stdout parseable. r[operator_diagnostics.project_soundness_diagnostics]
- [x] [serial] Integrate soundness issues with proof-before-claim wording in project readiness/evidence summaries. r[verification_evidence.project_soundness_claims]

## Verification

- [x] [serial] Add positive pure tests for clean project state, warning-only states, deterministic JSON issue ordering, and generated-input match. r[operator_diagnostics.project_soundness_diagnostics] r[project_workflows.project_soundness_checks]
- [x] [serial] Add negative pure tests for every diagnostic class, stale generated inputs, kind mismatch, unsupported hash algorithm, fetch policy conflict, trust failure facts, retention root mismatch, and no-network classification. r[project_workflows.project_soundness_checks]
- [x] [serial] Add CLI tests proving default check does not execute network probes, JSON output is parseable, and explicit probe/trust modes label behavior. r[operator_diagnostics.project_soundness_diagnostics]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[operator_diagnostics.project_soundness_diagnostics]
