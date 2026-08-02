# Tasks

## Phase 1: Source and policy admission

- [x] [depends:compile-foreign-derivation-graphs] I1 Add pure validation for executable plans, selected roots, import-receipt links, source requirements, and execution-profile bindings. r[foreign_derivation_import.realization_adapter]
- [x] [serial] I2 Add a typed Nickel foreign execution-profile contract with deterministic runtime export and bounded profile fields. r[foreign_derivation_import.execution_profile]
- [x] [serial] I3 Bind the canonical execution-profile BLAKE3 into each target derivation identity through a reserved internal field. Reject reserved-key collisions and digest mismatch. r[foreign_derivation_import.execution_profile]
- [x] [parallel] I4 Add positive profile tests and negative unknown-field, limit, collision, stale-digest, `/bin/sh`, network, syscall, writable-prefix, and environment tests. r[foreign_derivation_import.execution_profile]

## Phase 2: Source materialization and fetch policy

- [x] [serial] I5 Bind non-derivation source requirements to verified source-bundle records. Materialize them through castore and signed PathInfo without reading ambient foreign store paths. r[foreign_derivation_import.source_materialization]
- [x] [serial] I6 Extend fetch requests with bounded ordered candidates for compiled download nodes. Preserve fixed-output verification and fail closed on content mismatch. r[foreign_derivation_import.source_materialization]
- [x] [parallel] I7 Add file, directory, symlink, executable, fixed-output, Git, ordered-fallback, missing-record, digest-tamper, incomplete-tree, and wrong-mode tests. r[foreign_derivation_import.source_materialization]

## Phase 3: Scheduler and sandbox adapter

- [x] [serial] I8 Refactor build-request creation to require an explicit execution profile. Pass the current compatibility profile from existing callers. r[foreign_derivation_import.execution_profile]
- [x] [serial] I9 Register resolved units in `DerivationRegistry` with verified profile bindings, then realize selected roots through the ordinary `Builder` worker path. r[foreign_derivation_import.realization_adapter]
- [x] [serial] I10 Add the thin `mantle foreign-import realize` shell with explicit graph, plan, receipt, source, profile, store, state, and substitution inputs. r[foreign_derivation_import.realization_adapter]
- [x] [parallel] I11 Add local realization tests for fixed-output roots, exact two-node dependency paths, no-`/bin/sh` Guix profiles, substitution, sibling failure, cancellation, and unsupported remote execution. r[foreign_derivation_import.realization_adapter]

## Phase 4: Reports, documentation, and boundaries

- [x] [serial] I12 Emit `mantle-foreign-realization-receipt-v1` with source, profile, build-report, PathInfo, disposition, failure, strongest-state, and non-claim facts. r[foreign_derivation_import.realization_receipt]
- [x] [parallel] I13 Update trust-model, operator, source-bundle, and machine-artifact documentation. Preserve the OnixOS system-assembly boundary and realization non-claims. r[foreign_derivation_import.realization_receipt]
- [x] [serial] V1 Run `nix develop -c check-nickel-configs`, `nix develop -c cargo test -p crunch-build build_request::`, `nix develop -c cargo test -p crunch-build fetch_build_service::`, and `nix develop -c cargo test -p mantle --test foreign_import_cli`. Record exact output in `cairn/changes/realize-foreign-derivation-adapter/evidence/verification.md`. r[foreign_derivation_import.realization_adapter]
- [x] [serial] V2 Run the two-node local realization with `--no-substitute`, run the fixed-output mismatch fixture, and run the Guix profile no-`/bin/sh` fixture. Record the exact commands, exit statuses, build report identities, and a short inline evidence summary. r[foreign_derivation_import.realization_receipt]
- [x] [serial] V3 Run `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs`, its `--self-test`, `git diff --check`, focused first-party formatting and Clippy, Cairn validation, all three gates, and Tracey coverage. Record exact output before archive. r[foreign_derivation_import.realization_receipt]
