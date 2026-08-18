# Tasks

## 1. Contracts and identity

- [x] [serial] 1.1 Add versioned Nickel contracts for comparison policy, base and head snapshots, package outcome records, closure facts, and `mantle-package-impact-v1`. r[mantlepkgs_impact.compatible_snapshots]
- [x] [serial] 1.2 Add deterministic BLAKE3 identity helpers for policy, snapshot, per-package evidence, and complete reports. r[mantlepkgs_impact.functional_core]
- [x] [serial] 1.3 Add named limits for packages, variants, closure members, dependency edges, observations, diagnostics, and report bytes. r[mantlepkgs_impact.functional_core]

## 2. Functional comparison core

- [x] [serial] 2.1 Implement pure base and head snapshot validation with exact global compatibility checks. r[mantlepkgs_impact.compatible_snapshots]
- [x] [serial] 2.2 Implement deterministic package classification for added, removed, unchanged, recipe-changed, policy-changed, blocker-changed, and variant-changed records. r[mantlepkgs_impact.catalog_dispositions]
- [x] [serial] 2.3 Implement build-outcome transitions over admitted observations without inferring success from paths, CAS presence, or missing data. r[mantlepkgs_impact.outcome_transitions]
- [x] [serial] 2.4 Implement complete-closure deltas for members, retained dependencies, member count, and logical byte size. r[mantlepkgs_impact.closure_deltas]
- [x] [serial] 2.5 Implement stable non-comparable closure reasons that omit numeric regression claims. r[mantlepkgs_impact.closure_deltas]
- [x] [serial] 2.6 Implement canonical report ordering and identity that are independent of input order and host paths. r[mantlepkgs_impact.functional_core]

## 3. Evidence adapters and CLI

- [x] [serial] 3.1 Add a thin shell adapter over stored Mantle artifacts and PathInfo closure facts. r[mantlepkgs_impact.closure_deltas]
- [x] [serial] 3.2 Query only action results that pass current request, policy, platform, signature, output-set, PathInfo, CAS, and action admission. r[mantlepkgs_impact.outcome_transitions]
- [x] [serial] 3.3 Add `mantle mantlepkgs impact --base ... --head ... --out ...` with bounded local inputs and atomic publication. r[mantlepkgs_impact.functional_core]
- [x] [parallel] 3.4 Add deterministic external-CI fixture inputs and a pure forge-neutral adapter example. r[mantlepkgs_impact.external_ci_boundary]
- [x] [parallel] 3.5 Document that forge credentials, webhooks, comments, approvals, and publication effects remain outside Mantle. r[mantlepkgs_impact.external_ci_boundary]

## 4. Verification

- [x] [parallel] 4.1 Add unit tests for package classes, explicit build states, stale or rejected observations, and missing observations. r[mantlepkgs_impact.catalog_dispositions] r[mantlepkgs_impact.outcome_transitions]
- [x] [parallel] 4.2 Add closure tests for additions, removals, retained dependencies, missing PathInfo, incomplete closures, byte deltas, and semantic incompatibility. r[mantlepkgs_impact.closure_deltas]
- [x] [parallel] 4.3 Add adversarial tests for duplicate records, conflicting keys, malformed digests, limit excess, overflow, and partial evidence. r[mantlepkgs_impact.functional_core]
- [x] [parallel] 4.4 Add property and mutation tests for order independence, report identity stability, and identity sensitivity. r[mantlepkgs_impact.functional_core]
- [x] [parallel] 4.5 Add incompatible-system, store-prefix, and conversion-policy fixtures and verify no report is emitted. r[mantlepkgs_impact.compatible_snapshots]
- [x] [parallel] 4.6 Verify offline report generation without Nix and without network effects. r[mantlepkgs_impact.functional_core] r[mantlepkgs_impact.external_ci_boundary]
- [x] [parallel] 4.7 Add machine-readable schema, positive fixtures, negative fixtures, and freshness coverage for the new report family. r[mantlepkgs_impact.claim_boundary]

## 5. Lifecycle

- [x] [serial] 5.1 Run focused Rust, Nickel, schema, formatting, Clippy, and Tiger Style checks. r[mantlepkgs_impact.functional_core]
- [x] [serial] 5.2 Record bounded validation evidence and explicit non-claims. r[mantlepkgs_impact.claim_boundary]
- [x] [serial] 5.3 Run proposal, design, tasks, repository-validation, and traceability gates. r[mantlepkgs_impact.claim_boundary]
- [x] [serial] 5.4 Sync the accepted specification and archive the completed change. r[mantlepkgs_impact.claim_boundary]
