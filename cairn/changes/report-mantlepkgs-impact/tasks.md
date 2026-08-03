# Tasks: report Mantlepkgs impact

## Phase 1: Baseline and contracts

- [ ] [serial] I1 Run existing Mantlepkgs catalog, build-report, closure, action-result, and machine-contract tests before core changes. r[mantlepkgs_impact.functional_core]
- [ ] [serial] I2 Define versioned base snapshot, head snapshot, comparison policy, reason-code, and `mantle-package-impact-v1` contracts. r[mantlepkgs_impact.compatible_snapshots]
- [ ] [serial] I3 Add named limits for packages, variants, closure members, dependency edges, observations, diagnostics, and report bytes. r[mantlepkgs_impact.functional_core]
- [ ] [parallel] I4 Add positive compatible snapshots and negative system, prefix, policy, identity, digest, and limit fixtures. r[mantlepkgs_impact.compatible_snapshots]

## Phase 2: Pure catalog and outcome comparison

- [ ] [serial] I5 Add a pure deterministic join over package and variant identities. r[mantlepkgs_impact.catalog_dispositions]
- [ ] [serial] I6 Derive added, removed, unchanged, recipe-changed, policy-changed, blocker-changed, and variant-changed dispositions. r[mantlepkgs_impact.catalog_dispositions]
- [ ] [serial] I7 Derive explicit success, failure, blocked, not-attempted, and unavailable transitions without inventing observations. r[mantlepkgs_impact.outcome_transitions]
- [ ] [parallel] I8 Add positive transition tests and negative missing-observation, duplicate-key, conflicting-result, stale-result, and order tests. r[mantlepkgs_impact.outcome_transitions]

## Phase 3: Closure and dependency deltas

- [ ] [serial] I9 Add pure closure comparison for matching complete closure semantics. r[mantlepkgs_impact.closure_deltas]
- [ ] [serial] I10 Report added and removed members, member-count delta, logical byte delta, and retained dependencies. r[mantlepkgs_impact.closure_deltas]
- [ ] [parallel] I11 Add negative incomplete-closure, mismatched-size, missing-PathInfo, overflow, and over-limit fixtures. r[mantlepkgs_impact.closure_deltas]
- [ ] [parallel] I12 Prove incompatible closures emit non-comparability reasons instead of numeric regression claims. r[mantlepkgs_impact.closure_deltas]

## Phase 4: Shell and external adapter boundary

- [ ] [serial] I13 Add a Mantlepkgs CLI action that reads contracted artifacts, verifies digests, and writes one atomic report. r[mantlepkgs_impact.external_ci_boundary]
- [ ] [serial] I14 Query only action results that pass current Mantle request, policy, platform, signature, output, and CAS admission. r[mantlepkgs_impact.outcome_transitions]
- [ ] [parallel] I15 Add an external adapter fixture that consumes the report without giving forge credentials or webhook behavior to Mantle. r[mantlepkgs_impact.external_ci_boundary]
- [ ] [parallel] I16 Document report non-claims and exact comparability requirements. r[mantlepkgs_impact.claim_boundary]

## Phase 5: Validation and lifecycle

- [ ] [serial] V1 Run focused pure-core, CLI, closure, action-result, machine-contract, and external-adapter fixture tests. Record exact output in `evidence/verification.md`. r[mantlepkgs_impact.functional_core]
- [ ] [serial] V2 Run formatting, focused Clippy, property tests, mutation tests, `git diff --check`, and the bounded live comparison rail. r[mantlepkgs_impact.claim_boundary]
- [ ] [serial] V3 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, and relevant Nix checks. r[mantlepkgs_impact.external_ci_boundary]
