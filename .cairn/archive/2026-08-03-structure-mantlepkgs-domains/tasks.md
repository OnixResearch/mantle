# Tasks: structure Mantlepkgs package domains

## Phase 1: Baseline and contracts

- [x] [serial] I1 Run the existing Mantlepkgs core, CLI, machine-contract, and no-Nix consumer tests before core changes. r[mantlepkgs_domains.functional_core]
- [x] [serial] I2 Add typed Nickel contracts for shard identity, class, source lock, owner label, package selectors, variants, validation roots, and named limits. r[mantlepkgs_domains.domain_partition]
- [x] [serial] I3 Add versioned compatibility handling for existing single-domain catalogs. r[mantlepkgs_domains.domain_partition]
- [x] [parallel] I4 Add positive complete-shard fixtures and negative missing-field, invalid-class, duplicate-identity, and limit fixtures. r[mantlepkgs_domains.domain_partition]

## Phase 2: Pure composition and variants

- [x] [serial] I5 Add a pure deterministic composition core with canonical ordering and BLAKE3 identities. r[mantlepkgs_domains.deterministic_composition]
- [x] [serial] I6 Reject selector, alias, shard, package, and artifact conflicts without precedence or last-writer-wins behavior. r[mantlepkgs_domains.deterministic_composition]
- [x] [serial] I7 Add explicit variant records that bind base package identity, variant policy, root identity, and provenance. r[mantlepkgs_domains.explicit_variants]
- [x] [parallel] I8 Add positive multi-shard and variant tests plus negative collision, missing-base, cycle, tamper, and ordering tests. r[mantlepkgs_domains.deterministic_composition] r[mantlepkgs_domains.explicit_variants]

## Phase 3: Separate validation roots

- [x] [serial] I9 Add pure validation-root planning over package outputs, validation inputs, tools, policies, limits, and expected outcomes. r[mantlepkgs_domains.separate_validation_roots]
- [x] [serial] I10 Add a shell adapter that realizes validation roots without changing package recipe or output identity. r[mantlepkgs_domains.separate_validation_roots]
- [x] [parallel] I11 Add positive validation evidence and negative missing-output, stale-policy, timeout, malformed-result, and failed-validation evidence. r[mantlepkgs_domains.separate_validation_roots]
- [x] [parallel] I12 Prove that test-only source and dependency changes leave the package output identity unchanged. r[mantlepkgs_domains.separate_validation_roots]

## Phase 4: External corpus

- [x] [serial] I13 Record one exact `corepkgs` revision, repository identity, observed license, selected package set, and producer policy. r[mantlepkgs_domains.reference_corpus]
- [x] [serial] I14 Evaluate the pinned corpus through the foreign producer boundary and retain complete graph, source, catalog, and blocker evidence. r[mantlepkgs_domains.reference_corpus]
- [x] [parallel] I15 Add negative stale-lock, wrong-revision, license-record, artifact-tamper, unsupported-package, and missing-source fixtures. r[mantlepkgs_domains.reference_corpus]
- [x] [parallel] I16 Document that corpus success does not prove package correctness, broad compatibility, or release eligibility. r[mantlepkgs_domains.claim_boundary]

## Phase 5: Validation and lifecycle

- [x] [serial] V1 Run `cargo test -p mantlepkgs-core` and focused Mantlepkgs CLI integration tests. Record exact output in `evidence/verification.md`. r[mantlepkgs_domains.functional_core]
- [x] [serial] V2 Run host and no-Nix consumer tests, Nickel contract tests, machine-contract tests, formatting, focused Clippy, and `git diff --check`. r[mantlepkgs_domains.reference_corpus]
- [x] [serial] V3 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, and relevant Nix checks. r[mantlepkgs_domains.claim_boundary]
