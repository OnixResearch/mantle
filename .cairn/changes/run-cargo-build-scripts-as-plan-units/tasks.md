# Tasks: Run Cargo build scripts as plan units

All tasks remain open. Creating this proposal does not accept build-script
support.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Record the baseline in an isolated worktree: unit graphs and `rust-plan` build-script receipts for `proc-macro2`, `serde` with derive, `libc`, and one `-sys` fixture, plus the current lane blockers. r[mantle.rust_unit_plan.build_scripts.run_units]
- [ ] [serial] T1.2 Define the supported directive set, propagation rules, `flags` file formats, the target-facts format, and the native-input Nickel contract with role markers. r[mantle.rust_unit_plan.build_scripts.directive_propagation] r[mantle.rust_unit_plan.build_scripts.native_inputs]

## Phase 2: Core and lowering

- [ ] [serial] T2.1 Move directive parsing into the pure core with the enumerated directive set and typed failures. r[mantle.rust_unit_plan.build_scripts.directive_propagation]
- [ ] [serial] T2.2 Lower target-description units and build-script execution units with declared environments and `flags` references in dependents. r[mantle.rust_unit_plan.build_scripts.target_facts] r[mantle.rust_unit_plan.build_scripts.run_units]
- [ ] [serial] T2.3 Resolve native-input roles in the producer and add them as unit inputs. r[mantle.rust_unit_plan.build_scripts.native_inputs]
- [ ] [parallel] T2.4 Add core and lowering fixtures: directive parsing, transitive link propagation, `links` metadata, and shared target units. r[mantle.rust_unit_plan.build_scripts.directive_propagation] r[mantle.rust_unit_plan.build_scripts.target_facts]
- [ ] [parallel] T2.5 Add negative fixtures: an unknown directive, an `error` directive, an undeclared role marker, and duplicate `links` keys. r[mantle.rust_unit_plan.build_scripts.directive_propagation] r[mantle.rust_unit_plan.build_scripts.native_inputs]

## Phase 3: Helper and sandbox

- [ ] [serial] T3.1 Add target-facts and run-build-script modes to the unit helper, writing typed `flags` outputs. r[mantle.rust_unit_plan.build_scripts.run_units]
- [ ] [serial] T3.2 Consume `flags` in dependent compilations through argument and environment files. r[mantle.rust_unit_plan.build_scripts.directive_propagation]
- [ ] [parallel] T3.3 Add sandbox negative controls: a network attempt, a host-path probe, a nonzero exit, and a removed native declaration. r[mantle.rust_unit_plan.build_scripts.sandbox_boundary]

## Phase 4: Verification and documentation

- [ ] [serial] T4.1 Build `proc-macro2`, `serde` with derive, and `libc` graphs through the lane. r[mantle.rust_unit_plan.build_scripts.run_units]
- [ ] [serial] T4.2 Build a `-sys` crate against a declared Mantle-built library and prove a native-input change reruns only affected units. r[mantle.rust_unit_plan.build_scripts.native_inputs]
- [ ] [serial] T4.3 Update the matrix rows with named fixtures and extend the drift rail. r[mantle.rust_unit_plan.build_scripts.matrix_evidence]
- [ ] [serial] T4.4 Document build-script support, directives, native declarations, and non-claims in the lane documentation. r[mantle.rust_unit_plan.build_scripts.sandbox_boundary]
- [ ] [serial] T4.5 Run focused core, lowering, helper, and lane tests, `rust_compatibility_rail`, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.rust_unit_plan.build_scripts.run_units]
- [ ] [serial] T4.6 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.rust_unit_plan.build_scripts.matrix_evidence]
