# Tasks: Resolve Mantlepkgs version revisions

## Phase 1: Baseline and contracts

- [x] [serial] I1 Run focused Mantlepkgs manifest, domain composition, update-plan, and CLI tests before core changes. r[mantlepkgs_versions.functional_core]
- [x] [serial] I2 Add ADR 0076 and record the upstream design, rejected direct adoption, ownership boundary, and non-claims. r[mantlepkgs_versions.claim_boundary]
- [x] [serial] I3 Add typed Nickel contracts for cohorts, observations, indexes, requests, selection policy, receipts, groups, and named limits. r[mantlepkgs_versions.typed_index]
- [x] [parallel] I4 Add positive contracts and negative floating-cohort, wrong-system, duplicate-key, unknown-method, unsafe-path, and limit fixtures. r[mantlepkgs_versions.typed_index]

## Phase 2: Observation and index production

- [x] [serial] I5 Add pure observation validation and deterministic compact-index construction in `mantlepkgs-core`. r[mantlepkgs_versions.observation_status] r[mantlepkgs_versions.functional_core]
- [x] [serial] I6 Add a bounded producer shell for declared channel revisions, systems, and attribute allowlists. r[mantlepkgs_versions.observation_status]
- [x] [serial] I7 Require the package `version` attribute and record absent, malformed, failed, and unavailable observations without name parsing. r[mantlepkgs_versions.observation_status]
- [x] [parallel] I8 Add positive multi-revision observations and negative missing-attribute, missing-version, evaluation-failure, hash-mismatch, and oversized fixtures. r[mantlepkgs_versions.observation_status]
- [x] [parallel] I9 Audit one pinned `nixpkgs-multiverse` revision as comparison evidence with repository, revision, license observation, system, and artifact digest. r[mantlepkgs_versions.claim_boundary]

## Phase 3: Pure resolution and receipts

- [x] [serial] I10 Add pure request resolution for `newest-published-revision-for-reported-version` with stable blocker codes. r[mantlepkgs_versions.deterministic_resolution]
- [x] [serial] I11 Add deterministic grouping by exact selected revision and versioned public selector validation. r[mantlepkgs_versions.revision_grouping]
- [x] [serial] I12 Add `mantlepkgs-version-resolution-v1` receipts with BLAKE3 identity and tagged Nix SHA-256 `narHash`. r[mantlepkgs_versions.resolution_receipt]
- [x] [parallel] I13 Add property tests for input-order independence, newest-revision selection, grouping, and canonical identity. r[mantlepkgs_versions.deterministic_resolution] r[mantlepkgs_versions.functional_core]
- [x] [parallel] I14 Add negative unknown-version, cross-system, stale-index, ambiguous-selector, changed-policy, changed-revision, and changed-hash tests. r[mantlepkgs_versions.resolution_receipt]

## Phase 4: Producer recheck and Mantlepkgs composition

- [x] [serial] I15 Materialize each selected source, recheck the target attribute version, and compute source-tree BLAKE3 before generation. r[mantlepkgs_versions.producer_recheck]
- [x] [serial] I16 Emit one existing Mantlepkgs manifest per accepted revision group without changing v1 source-lock meaning. r[mantlepkgs_versions.revision_grouping]
- [x] [serial] I17 Adapt generated catalogs into domain shards and compose distinct versioned selectors. r[mantlepkgs_versions.revision_grouping]
- [x] [parallel] I18 Prove recheck failures never select another revision or publish a partial success generation. r[mantlepkgs_versions.producer_recheck]
- [x] [parallel] I19 Add a pinned `x86_64-linux` pilot with two reported versions of one leaf package. r[mantlepkgs_versions.claim_boundary]

## Phase 5: Documentation and validation

- [x] [serial] I20 Document index updates, resolution, recheck, generation grouping, no-Nix consumption, and bounded claims. r[mantlepkgs_versions.claim_boundary]
- [x] [serial] V1 Run focused core, producer-shell, receipt, grouping, domain, and CLI tests after implementation. r[mantlepkgs_versions.functional_core]
- [x] [parallel] V2 Run positive and negative Nickel fixtures, property tests, formatting, focused Clippy, and `git diff --check`. r[mantlepkgs_versions.typed_index] r[mantlepkgs_versions.resolution_receipt]
- [x] [serial] V3 Run the pilot with exact revision, `narHash`, version recheck, generation, and domain-composition evidence. r[mantlepkgs_versions.producer_recheck] r[mantlepkgs_versions.revision_grouping]
- [x] [serial] V4 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, and relevant Nix checks. r[mantlepkgs_versions.claim_boundary]
