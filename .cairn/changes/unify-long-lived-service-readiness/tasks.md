# Tasks: One readiness vocabulary for long-lived Mantle services

Only the bounded contract-definition tasks T1.2 and T1.3 are recorded as complete; all other tasks remain open. This is not producer acceptance or archive readiness.

## Phase 1: Contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current readiness behavior of the Rust cache daemon, remote serve, and proof stages, including one blocked-dependency observation. r[mantle.service_readiness.readiness_vocabulary]
- [x] [serial] T1.2 Define the state vocabulary, dependency declaration shape, and restart policy matrix as a versioned schema. r[mantle.service_readiness.readiness_vocabulary]
- [x] [serial] T1.3 Record the `ready`-additional, declared-dependency, and closed-matrix decisions in an ADR. r[mantle.service_readiness.restart_policy_matrix]

## Phase 2: Core

- [ ] [serial] T2.1 Implement pure vocabulary validation, dependency graph evaluation, restart policy normalization, and derived state computation. r[mantle.service_readiness.declared_dependencies]
- [ ] [serial] T2.2 Add the readiness reporting boundary and the blocked-dependency result. r[mantle.service_readiness.declared_dependencies]

## Phase 3: Consumers

- [ ] [serial] T3.1 Report `started` and `ready` separately for the coordination daemon, including restart, and name its restart policy. r[mantle.service_readiness.readiness_vocabulary]
- [ ] [serial] T3.2 Report `started` and `ready` separately for the Rust cache daemon, and name its restart policy. r[mantle.service_readiness.restart_policy_matrix]
- [ ] [serial] T3.3 Report `started` and `ready` separately for the remote serve binding, and name its restart policy. r[mantle.service_readiness.restart_policy_matrix]
- [ ] [serial] T3.4 Declare dependency relations for the source-built fixed-point proof stages and report each stage's state under the vocabulary. r[mantle.service_readiness.declared_dependencies]
- [ ] [serial] T3.5 Make `mantle doctor` publish derived readiness state while keeping existing human and JSON output unchanged. r[mantle.service_readiness.doctor_derived_state]

## Phase 4: Fixtures and verification

- [ ] [parallel] T4.1 Add positive fixtures: `ready` only after a real request succeeds; a dependent component starts after its dependency is `ready`; each restart policy observed on a controlled exit. r[mantle.service_readiness.readiness_vocabulary]
- [ ] [parallel] T4.2 Add negative fixtures: process exits before readiness reports `failed`, not `ready`; blocked dependency reports the blocker; unknown state rejected; missing policy rejected. r[mantle.service_readiness.declared_dependencies]
- [ ] [parallel] T4.3 Prove doctor evidence separation: derived state is not accepted by evidence validators and does not change receipts. r[mantle.service_readiness.doctor_derived_state]
- [ ] [serial] T4.4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[mantle.service_readiness.readiness_vocabulary]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.service_readiness.doctor_derived_state]
