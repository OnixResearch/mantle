# Tasks: preserve Nix fetch mirror order

## Phase 1: Baseline and canonical model

- [x] [serial] V1 Run the current focused producer, compiler, fetch-service, and CLI tests before core changes. Record exact output in `evidence/baseline.md`. r[foreign_derivation_import.nix_fetch_candidate_normalization]
  - Evidence: `evidence/baseline.md` records the task-specified filters, the corrected binary-target baseline from `origin/main` (`20` producer and `18` compiler tests), `23` fetch-service tests, and `16` passing CLI tests with one ignored external parity test.
- [x] [serial] I1 Add a bounded ordered candidate field to supported foreign fixed-output fetch nodes. Preserve decoding for existing graph artifacts without that field. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [serial] I2 Add canonical validation for candidate count, order, URL support, duplicates, legacy agreement, and reserved private fields. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [parallel] I3 Add positive and negative graph-model tests for canonical, legacy, conflicting, invalid, duplicate, oversized, and reserved-field cases. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 2: Pure Nix candidate normalization

- [x] [serial] I4 Add a pure normalization core for direct `url`, unstructured `urls`, and bounded structured `__json` candidate facts. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [serial] I5 Treat structured candidate fields as authoritative. Reject mixed structured and top-level candidate fields, and enforce matching primary values within one representation. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [serial] I6 Reject unresolved `mirror://` aliases and unsupported schemes unless the producer emits explicit expanded candidates with provenance. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [parallel] I7 Add pure-core tests for singleton, unstructured, structured, matching, whitespace-only, malformed, non-string, mixed-surface, conflicting, alias, and limit cases. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 3: Nix producer and compiler integration

- [x] [serial] I8 Thread canonical candidate extraction through direct ATerm and derivation-JSON lowering with equal graph output for equal concrete facts. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [serial] I9 Keep candidate extraction behind supported fixed-output fetch classification. Do not infer simple-download semantics from a hash or candidate field alone. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [serial] I10 Compile canonical candidates into `url` plus the private ordered-candidate binding. Bind the exact order into target derivation and executable-plan identity. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [parallel] I11 Add ATerm, derivation-JSON, compiler, plan-snapshot, and CLI fixtures that prove exact candidate order and Nix-free consumption. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 4: Fetch behavior and evidence

- [x] [serial] I12 Keep the fetch service frontend-neutral. It must consume only the canonical compiler output, not Nix `urls`, `__json`, or mirror tables. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [serial] I13 Preserve sequential fallback for transport, Git, and I/O failures. Preserve terminal fixed-output mismatch behavior and PathInfo rejection. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [parallel] I14 Add tests for first-candidate failure, second-candidate selection, exhausted candidates, attempt ordering, fixed-output mismatch, and no PathInfo admission. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [x] [parallel] I15 Update foreign import trust and realization documents. State supported forms, blockers, identity effects, and non-claims. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 5: Validation and lifecycle

- [x] [serial] V2 Run `nix develop -c cargo test -p mantle --lib foreign_derivation_import::`, `nix develop -c cargo test -p mantle --lib foreign_graph_compiler::`, `nix develop -c cargo test -p crunch-build --lib fetch_build_service::`, and `nix develop -c cargo test -p mantle --test foreign_import_cli`. Record exact output in `evidence/verification.md`. r[foreign_derivation_import.nix_fetch_candidate_normalization]
  - Evidence: `evidence/verification.md` records the exact commands. The task-specified root-lib filters select zero tests, so the corrected binary-target runs prove `27` producer and `19` compiler tests. The fetch-service run proves `24` tests. The CLI run proves `16` tests with one ignored external parity test.
- [x] [serial] V3 Run `nix develop -c cargo fmt --check -p mantle -p crunch-build`, `nix develop -c ./scripts/check-first-party-clippy.sh`, `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs`, `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test`, and `git diff --check`. Record exact output in `evidence/verification.md`. r[foreign_derivation_import.nix_fetch_candidate_normalization]
  - Evidence: `evidence/quality.md` records passing changed-file Rustfmt, first-party Clippy, trust-model, self-test, and diff checks. The task-wide format command retains only the unchanged excluded `src/source_built_fixed_point_shell.rs` difference; `git diff --exit-code origin/main --` proves this change did not modify that file.
- [x] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, all three Cairn gates, and `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .`. Then sync, archive, rerun post-archive validation, and retain the exact transcript. r[foreign_derivation_import.nix_fetch_candidate_normalization]
  - Evidence: `evidence/pre-archive-validation.json`, `proposal-gate.json`, `design-gate.json`, `tasks-gate.json`, and `pre-archive-tracey.log` record valid pre-archive results. Post-archive evidence is retained with the archived package.
