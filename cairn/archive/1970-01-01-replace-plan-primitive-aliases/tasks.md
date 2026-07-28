# Tasks: Replace dynamic-plan primitive aliases

## Phase 1: Baseline and model split

- [x] [serial] V1 Capture current `mantle-plan-v1` JSON, canonical bytes, plan digests, placeholder results, graph diagnostics, and public Rust call sites. r[build_correctness.dynamic_plan_nominal.compatibility]
- [x] [serial] I1 Add current-shape wire DTOs and a pure wire-to-admitted-plan conversion boundary. r[build_correctness.dynamic_plan_nominal.wire_boundary]
- [x] [serial] I2 Add private `UnitId`, `SourceId`, `StorePathString`, and `OutputName` newtypes with existing limits, checked constructors, and explicit accessors. r[build_correctness.dynamic_plan_nominal.values]
- [x] [parallel] V2 Add constructor tests for valid, empty, oversized, control-bearing, malformed, and store-prefix-invalid inputs. r[build_correctness.dynamic_plan_nominal.values.validation]

## Phase 2: Digest and graph migration

- [x] [serial] I3 Add private BLAKE3 storage with separate `PlanDigest` and `NarDigest` marker aliases. r[build_correctness.dynamic_plan_nominal.digests]
- [x] [serial] I4 Migrate placeholders, roots, sources, units, inputs, requested outputs, and graph indexes to typed values without early string conversion. r[build_correctness.dynamic_plan_nominal.graph]
- [x] [serial] I5 Project admitted plans back to the exact versioned wire shape for canonical serialization and hashing. r[build_correctness.dynamic_plan_nominal.compatibility]
- [x] [parallel] I6 Update plan producers and consumers through explicit compatibility adapters. r[build_correctness.dynamic_plan_nominal.wire_boundary]

## Phase 3: Evidence

- [x] [parallel] V3 Add compile-pass fixtures for same-domain calls and compile-fail fixtures for unit/source, plan/NAR digest, path/source, and output/unit substitution. r[build_correctness.dynamic_plan_nominal.compile_time]
- [x] [parallel] V4 Add positive and negative wire admission, placeholder, graph-reference, duplicate-ID, and wrong-role tests. r[build_correctness.dynamic_plan_nominal.graph.validation]
- [x] [parallel] V5 Prove canonical JSON and plan BLAKE3 identities remain unchanged for the accepted fixture cohort. r[build_correctness.dynamic_plan_nominal.compatibility]
- [x] [parallel] V6 Run focused dynamic-plan, planner, topology, and producer compatibility tests. r[build_correctness.dynamic_plan_nominal.graph]

## Phase 4: Adoption and closeout

- [x] [serial] I7 Add Mantle nominal-domain declarations for the Octet policy after the enforcement lint is available. r[build_correctness.dynamic_plan_nominal.octet]
- [x] [serial] I8 Document wire/core admission, new Rust APIs, store-presence non-claims, and consumer migration. r[build_correctness.dynamic_plan_nominal.docs]
- [x] [serial] V7 Run focused tests, first-party quality wrappers, Cairn validation, change gates, and relevant Nix checks. r[build_correctness.dynamic_plan_nominal.final_checks]

## Verification coverage

- `Scenario: Valid wire plan admits typed values` -> I1, I2, V2
- `Scenario: Invalid scalar fails before graph use` -> I1, I2, V2
- `Scenario: Unit and source IDs do not compile interchangeably` -> I2, V3
- `Scenario: Plan and NAR digests do not compile interchangeably` -> I3, V3
- `Scenario: Graph validation stays typed` -> I4, V4
- `Scenario: Wrong-role placeholder fails` -> I4, V4
- `Scenario: Canonical plan bytes remain stable` -> V1, I5, V5
- `Scenario: Existing producers retain wire compatibility` -> I6, V6
- `Scenario: Octet checks the migrated core` -> I7, V7
- `Scenario: Typed plan claims remain bounded` -> I8, V7
