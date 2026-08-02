# Tasks: preserve Nix fetch mirror order

## Phase 1: Baseline and canonical model

- [ ] [serial] V1 Run the current focused producer, compiler, fetch-service, and CLI tests before core changes. Record exact output in `evidence/baseline.md`. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I1 Add a bounded ordered candidate field to supported foreign fixed-output fetch nodes. Preserve decoding for existing graph artifacts without that field. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I2 Add canonical validation for candidate count, order, URL support, duplicates, legacy agreement, and reserved private fields. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [parallel] I3 Add positive and negative graph-model tests for canonical, legacy, conflicting, invalid, duplicate, oversized, and reserved-field cases. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 2: Pure Nix candidate normalization

- [ ] [serial] I4 Add a pure normalization core for direct `url`, unstructured `urls`, and bounded structured `__json` candidate facts. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I5 Treat structured candidate fields as authoritative. Reject mixed structured and top-level candidate fields, and enforce matching primary values within one representation. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I6 Reject unresolved `mirror://` aliases and unsupported schemes unless the producer emits explicit expanded candidates with provenance. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [parallel] I7 Add pure-core tests for singleton, unstructured, structured, matching, whitespace-only, malformed, non-string, mixed-surface, conflicting, alias, and limit cases. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 3: Nix producer and compiler integration

- [ ] [serial] I8 Thread canonical candidate extraction through direct ATerm and derivation-JSON lowering with equal graph output for equal concrete facts. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I9 Keep candidate extraction behind supported fixed-output fetch classification. Do not infer simple-download semantics from a hash or candidate field alone. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I10 Compile canonical candidates into `url` plus the private ordered-candidate binding. Bind the exact order into target derivation and executable-plan identity. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [parallel] I11 Add ATerm, derivation-JSON, compiler, plan-snapshot, and CLI fixtures that prove exact candidate order and Nix-free consumption. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 4: Fetch behavior and evidence

- [ ] [serial] I12 Keep the fetch service frontend-neutral. It must consume only the canonical compiler output, not Nix `urls`, `__json`, or mirror tables. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] I13 Preserve sequential fallback for transport, Git, and I/O failures. Preserve terminal fixed-output mismatch behavior and PathInfo rejection. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [parallel] I14 Add tests for first-candidate failure, second-candidate selection, exhausted candidates, attempt ordering, fixed-output mismatch, and no PathInfo admission. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [parallel] I15 Update foreign import trust and realization documents. State supported forms, blockers, identity effects, and non-claims. r[foreign_derivation_import.nix_fetch_candidate_normalization]

## Phase 5: Validation and lifecycle

- [ ] [serial] V2 Run `nix develop -c cargo test -p mantle --lib foreign_derivation_import::`, `nix develop -c cargo test -p mantle --lib foreign_graph_compiler::`, `nix develop -c cargo test -p crunch-build --lib fetch_build_service::`, and `nix develop -c cargo test -p mantle --test foreign_import_cli`. Record exact output in `evidence/verification.md`. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] V3 Run `nix develop -c cargo fmt --check -p mantle -p crunch-build`, `nix develop -c ./scripts/check-first-party-clippy.sh`, `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs`, `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test`, and `git diff --check`. Record exact output in `evidence/verification.md`. r[foreign_derivation_import.nix_fetch_candidate_normalization]
- [ ] [serial] V4 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, all three Cairn gates, and `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- tracey coverage --root .`. Then sync, archive, rerun post-archive validation, and retain the exact transcript. r[foreign_derivation_import.nix_fetch_candidate_normalization]
