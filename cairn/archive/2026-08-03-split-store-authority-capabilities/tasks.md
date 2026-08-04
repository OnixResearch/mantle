# Tasks: Split store authority into narrow capabilities

## Phase 1: Baseline and architecture

- [x] [serial] V1 Record current `StoreHandle`, `Builder`, pipeline, source-import, action-result, root, and administration call sites plus focused pre-change test output. r[build_correctness.store_capabilities.compatibility]
- [x] [serial] I1 Review and accept ADR 0058 for concrete capability values, private raw services, separate session state, and local claim boundaries. r[build_correctness.store_capabilities.roles] r[build_correctness.store_capabilities.claim_boundary]
- [x] [depends:persist-rust-unit-castore-results] [depends:share-rust-unit-action-results] I2 Reconcile new Rust-unit store and action-result call paths with the capability ownership table before implementation. r[build_correctness.store_capabilities.roles]

## Phase 2: Store capability shell

- [x] [serial] I3 Add private shared store internals and concrete `BuildStore`, `OutputLookup`, `RootRegistry`, `SourceAdmission`, `ActionResultPort`, and `StoreAdmin` values. r[build_correctness.store_capabilities.roles]
- [x] [serial] I4 Move build-session output nodes, built-output facts, substitution reports, and CA resolution into builder-owned session state or `BuildStore`. r[build_correctness.store_capabilities.roles]
- [x] [serial] I5 Add named high-level methods for closure resolution, castore transformation, NAR calculation, cache lookup, substitution, output admission, and fixed action-result exchange without returning writable service objects. r[build_correctness.store_capabilities.raw_service_confinement]

## Phase 3: Consumer migration

- [x] [serial] I6 Replace `Builder`'s `StoreHandle` field and remove `Builder::store_handle()`. r[build_correctness.store_capabilities.roles] r[build_correctness.store_capabilities.raw_service_confinement]
- [x] [serial] I7 Migrate `crunch-pipeline` to retained `OutputLookup` and `RootRegistry` values, including one bounded `register_if_present` operation. r[build_correctness.store_capabilities.roles]
- [x] [serial] I8 Migrate CLI, source-bundle, cache export, repair, and test wiring to the correct capabilities and keep any compatibility facade shell-owned. r[build_correctness.store_capabilities.raw_service_confinement]

## Phase 4: Positive and negative verification

- [x] [parallel] V2 Add positive tests for local builds, cache hits, remote substitution, CA outputs, shared action-result reuse and publication, selected root registration, and source import through the correct capability. r[build_correctness.store_capabilities.compatibility]
- [x] [parallel] V3 Add compile-fail examples and source-policy tests that reject GC, source import, root mutation, backend replacement, publisher replacement, and raw service access from build or pipeline code. r[build_correctness.store_capabilities.raw_service_confinement]
- [x] [parallel] V4 Add negative runtime tests proving rejected outputs cannot persist PathInfo, output bytes, attestations, roots, action results, or success reports. r[build_correctness.store_capabilities.compatibility]
- [x] [parallel] V5 Add golden or equivalent compatibility checks for accepted report fields, PathInfo facts, output paths, and BLAKE3 identities. r[build_correctness.store_capabilities.compatibility]

## Phase 5: Documentation and lifecycle

- [x] [serial] I9 Document the capability ownership table, compatibility facade, migration rules, and bounded claim. r[build_correctness.store_capabilities.claim_boundary]
- [x] [serial] V6 Run `nix develop -c cargo test -p crunch-store`, `nix develop -c cargo test -p crunch-build`, and `nix develop -c cargo test -p crunch-pipeline`. Record exact positive and negative summaries. r[build_correctness.store_capabilities.compatibility]
- [x] [serial] V7 Run focused formatting, first-party Clippy, `./scripts/check-first-party-tigerstyle.sh`, `./scripts/check-first-party-quality.sh`, Cairn validation, Tracey coverage, all three change gates, and relevant Nix checks. r[build_correctness.store_capabilities.raw_service_confinement] r[build_correctness.store_capabilities.claim_boundary]
