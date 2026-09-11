# Tasks: Standardize derivation finish gates

All implementation and acceptance tasks remain open. Proposal creation is not
producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current per-recipe smoke inventory under `bootstrap/`, the builder-layer finish surface, and focused baseline test output. r[mantle.derivation_finish_gates.shared_contract]
- [ ] [serial] T1.2 Define the versioned finish-gate contract in typed Nickel policy with the deterministic export, named bounds, defaults, and explicit opt-outs. r[mantle.derivation_finish_gates.shared_contract]
- [ ] [serial] T1.3 Review and accept the gate-set decision (version default-on, leak policy per build kind, relocation and dlopen opt-in) in an ADR. r[mantle.derivation_finish_gates.shared_contract]

## Phase 2: Core and shell

- [ ] [serial] T2.1 Implement pure gate evaluation over declared gate records and observed facts with typed denials and named bounds. r[mantle.derivation_finish_gates.shared_contract]
- [ ] [serial] T2.2 Implement the version gate in the finish shell: empty-environment run, declared command resolution, default `--version` command for bin-installing derivations. r[mantle.derivation_finish_gates.version_check]
- [ ] [serial] T2.3 Implement the reference leak gate: prefix scan over outputs, cross-build deny list from build-only dependencies, bounded context excerpts. r[mantle.derivation_finish_gates.reference_leak_gate]
- [ ] [serial] T2.4 Implement the relocation rerun gate as an opt-in finish step. r[mantle.derivation_finish_gates.relocated_rerun]
- [ ] [serial] T2.5 Implement the dlopen audit gate with a bounded loader-audit module for Linux dynamic outputs. r[mantle.derivation_finish_gates.dlopen_audit]
- [ ] [serial] T2.6 Surface gate results in the build report under a versioned block. r[mantle.derivation_finish_gates.shared_contract]

## Phase 3: Positive and negative verification

- [ ] [parallel] T3.1 Add positive fixtures: passing version output, clean leak scan, passing relocation rerun, declared optional soname. r[mantle.derivation_finish_gates.version_check] r[mantle.derivation_finish_gates.reference_leak_gate] r[mantle.derivation_finish_gates.relocated_rerun]
- [ ] [parallel] T3.2 Add negative fixtures: wrong version, nonzero exit, missing binary without opt-out, cross leak, native report hit, failed rerun, undeclared missing dlopen, oversized or malformed gate record. r[mantle.derivation_finish_gates.version_check] r[mantle.derivation_finish_gates.reference_leak_gate] r[mantle.derivation_finish_gates.relocated_rerun] r[mantle.derivation_finish_gates.dlopen_audit]
- [ ] [serial] T3.3 Prove existing recipes evaluate and build unchanged before any adoption commit. r[mantle.derivation_finish_gates.shared_contract]

## Phase 4: Adoption

- [ ] [serial] T4.1 Adopt the gates in the bootstrap recipe family with per-family review of opt-outs, starting with the GCC wrapper outputs that previously leaked build-tree paths. r[mantle.derivation_finish_gates.reference_leak_gate]
- [ ] [serial] T4.2 Run focused core and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.derivation_finish_gates.shared_contract]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.derivation_finish_gates.shared_contract]
