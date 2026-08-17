# Tasks: plan Mantlepkgs source updates

## Phase 1: Baseline and typed policy

- [x] [serial] I1 Run existing project-refresh, source-lock, Mantlepkgs, validation-root, and impact-report tests before core changes. r[mantlepkgs_updates.functional_core]
- [x] [serial] I2 Add typed Nickel contracts for source kind, current identity, version rules, patch policy, advisory policy, validation policy, and named limits. r[mantlepkgs_updates.typed_policy]
- [x] [serial] I3 Add policy versions, canonical BLAKE3 identity, and explicit migration handling. r[mantlepkgs_updates.typed_policy]
- [x] [parallel] I4 Add positive policies and negative missing-field, executable-fragment, invalid-rule, unsafe-path, stale-identity, and limit fixtures. r[mantlepkgs_updates.typed_policy]

## Phase 2: Source observations and candidate selection

- [x] [serial] I5 Define bounded source-observation records with explicit success, unavailable, and failed states. r[mantlepkgs_updates.source_observations]
- [x] [serial] I6 Add one Git-tag adapter and one saved directory or release observation adapter in the imperative shell. r[mantlepkgs_updates.source_observations]
- [x] [serial] I7 Add a pure deterministic core for version normalization, constraint filtering, candidate ordering, and reason codes. r[mantlepkgs_updates.candidate_selection]
- [x] [parallel] I8 Add positive version tests and negative malformed, ignored, prerelease, duplicate, empty, unavailable, failed, oversized, and ambiguous fixtures. r[mantlepkgs_updates.candidate_selection]

## Phase 3: Preimage-bound mutation

- [x] [serial] I9 Add pure effect planning for exact structured fields, old and new values, input and output digests, and reasons. r[mantlepkgs_updates.preimage_bound_mutation]
- [x] [serial] I10 Add no-mutate default output and explicit staged execution with preimage revalidation and no-clobber publication. r[mantlepkgs_updates.preimage_bound_mutation]
- [x] [parallel] I11 Add negative stale-preimage, duplicate-field, unsafe-path, symlink, output-digest, partial-write, and publication-conflict tests. r[mantlepkgs_updates.preimage_bound_mutation]
- [x] [parallel] I12 Prove failed execution leaves source files unchanged and retains a typed denial receipt. r[mantlepkgs_updates.preimage_bound_mutation]

## Phase 4: Advisory and validation evidence

- [x] [serial] I13 Add bounded OSV and Repology observation adapters with response identities and explicit status. r[mantlepkgs_updates.advisory_evidence]
- [x] [parallel] I14 Add positive no-finding and finding fixtures plus negative timeout, transport, status, schema, coordinate, digest, and limit fixtures. r[mantlepkgs_updates.advisory_evidence]
- [x] [serial] I15 Link candidate catalog, package build, separate validation roots, and package-impact report into one update receipt. r[mantlepkgs_updates.validation_evidence]
- [x] [parallel] I16 Prove missing or failed advisory and validation evidence never becomes an empty success result. r[mantlepkgs_updates.advisory_evidence] r[mantlepkgs_updates.validation_evidence]
- [x] [parallel] I17 Document advisory, package-trust, correctness, reproducibility, and release non-claims. r[mantlepkgs_updates.claim_boundary]

## Phase 5: Validation and lifecycle

- [x] [serial] V1 Run focused policy-core, source-adapter, mutation, advisory, validation-link, and CLI tests. Record exact output in `evidence/verification.md`. r[mantlepkgs_updates.functional_core]
- [x] [serial] V2 Run property tests, mutation tests, Nickel contract tests, formatting, focused Clippy, `git diff --check`, and an offline replay from saved observations. r[mantlepkgs_updates.validation_evidence]
- [x] [serial] V3 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, and relevant Nix checks. r[mantlepkgs_updates.claim_boundary]
