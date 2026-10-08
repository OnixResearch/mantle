# Tasks: Build Cargo units as dynamic-plan derivations

The bounded two-package implementation, signed same-state app proof, and typed wrong-root rejection are recorded. Unchecked baseline, broader work-reduction, final immutable-source proof gates, and archival remain open; this package is not accepted.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree: offline Cargo executions for unchanged and one-package-edit rebuilds of the representative fixtures, `rust-plan` receipts for the same fixtures, and Mantle's own unit count, maximum per-unit inputs, and estimated plan size. r[mantle.rust_unit_plan.work_reduction]
- [x] [serial] T1.2 Define the lane profile contract, producer inputs, lowering binding table, manifest format, helper interface, and blocker catalog. r[mantle.rust_unit_plan.producer] r[mantle.rust_unit_plan.supported_fragment]
- [x] [serial] T1.3 Review ADR 0081 against the implemented bounded contract and record its decision revision in `adr/` without claiming final build-lane acceptance. r[mantle.rust_unit_plan.frontend_boundary]

## Phase 2: Pure lowering

- [x] [serial] [depends:separate-rust-plan-hexagon] T2.1 Implement the lowering adapter over the core's explicit unit effects, with placeholders, input addressing, metadata from unit identity, and source remapping. r[mantle.rust_unit_plan.lowering]
- [x] [serial] T2.2 Lower direct dependency inputs and the manifest contract for library units. r[mantle.rust_unit_plan.dependency_closure]
- [x] [parallel] T2.3 Add lowering goldens: library, binary, and proc-macro units, dependency edges, remapping, metadata, and order-independent plan digests. r[mantle.rust_unit_plan.lowering]
- [x] [parallel] T2.4 Add negative lowering fixtures: a build-script execution unit, an unsupported mode, a git source, an undeclared triple, a host path, and over-limit units, inputs, and bytes. r[mantle.rust_unit_plan.supported_fragment]

## Phase 3: Helper, producer, and entry point

- [x] [serial] T3.1 Implement the static unit helper: environment-provided outputs, argument file, rustc invocation, manifest writing, and search paths from dependency manifests. r[mantle.rust_unit_plan.unit_helper] r[mantle.rust_unit_plan.dependency_closure]
- [x] [serial] [depends:add-dynamic-plan-source-slices] T3.2 Emit one source slice per package source root and reference only the owning slice from each unit. r[mantle.rust_unit_plan.per_crate_sources]
- [x] [serial] T3.3 Implement the sandboxed producer with offline oracle capture, unit-graph version checks, and the evidence record. r[mantle.rust_unit_plan.producer]
- [x] [serial] T3.4 Add the Nickel entry point and `unit_plan` matrix lane; bind the evidence class, build status, and non-claims in the producer evidence record, and accepted-plan facts in the JSON build report. r[mantle.rust_unit_plan.evidence_lane]
- [x] [parallel] T3.5 Add producer and helper negative controls: a network attempt, a missing registry source, an unknown unit-graph version, a missing manifest, and ambient Cargo variables. r[mantle.rust_unit_plan.producer] r[mantle.rust_unit_plan.unit_helper]

## Phase 4: Verification and documentation

- [ ] [serial] T4.1 Build the supported representative fixtures through `mantle build`, prove an unchanged warm rerun executes zero units, and compare the supported default-feature two-package app against default `offlineCargoPackage` on the same immutable source, lock, pinned toolchain, target, and profile. Run both apps with identical arguments and environment; check actual stdout, stderr, and exit against the same expected result, fail parity on mismatch, and leave unsupported or unavailable comparisons unproven. Do not claim raw binary parity. r[mantle.rust_unit_plan.frontend_boundary] r[mantle.rust_unit_plan.work_reduction]
- [ ] [serial] T4.2 Record the work-reduction bundle for fresh, one-package edit, outside-package edit, and clean-client runs. r[mantle.rust_unit_plan.work_reduction] r[mantle.rust_unit_plan.per_crate_sources]
- [x] [serial] T4.3 Measure Mantle's own workspace against plan and scheduler limits and record the counts without a pass or fail claim. r[mantle.rust_unit_plan.supported_fragment]
- [x] [serial] T4.4 Document the lane in `docs/operator-workflows.md`, the examples catalog, and the matrix, including non-claims and the relationship to `offlineCargoPackage` and `rust-plan`. r[mantle.rust_unit_plan.evidence_lane]
- [ ] [serial] T4.5 Run focused Rust-plan core, application, and helper tests, `examples_inventory` and `rust_compatibility_rail`, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.rust_unit_plan.lowering]
- [ ] [serial] T4.6 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.rust_unit_plan.evidence_lane]
