# Tasks: One readiness vocabulary for long-lived Mantle services

Contract decisions, bounded producer runtime, and fixture observations are
recorded. This does not establish source-built proof-stage completion,
authoritative receipt invariance, final gates, or archive eligibility.

## Phase 1: Contract

- [x] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current readiness behavior of the Rust cache daemon, remote serve, and proof stages, including one blocked-dependency observation (`evidence/baseline-2026-09-30.md`; no new readiness producer acceptance). r[mantle.service_readiness.readiness_vocabulary]
- [x] [serial] T1.2 Define the state vocabulary, dependency declaration shape, and restart policy matrix as a versioned schema (`specs/service-readiness/schema.json`, design contract; producer acceptance remains open). r[mantle.service_readiness.readiness_vocabulary]
- [x] [serial] T1.3 Record the `ready`-additional, declared-dependency, and closed-matrix decisions in Proposed ADR 0091 and its index row (implementation remains open). r[mantle.service_readiness.restart_policy_matrix]

## Phase 2: Core

- [x] [serial] T2.1 Implement pure vocabulary validation, dependency graph evaluation, restart policy normalization, and derived state computation in `crates/crunch-service-readiness-core` (root workspace registration and consumer adapters now exist in source; T3/T4 runtime acceptance remains pending; original scoped proof `evidence/t2-core-2026-10-01.md`). r[mantle.service_readiness.declared_dependencies]
- [x] [serial] T2.2 Add the bounded non-evidence readiness reporting boundary and direct blocked-dependency result (`evidence/t2-core-2026-10-01.md`; no real daemon acknowledgment claimed). r[mantle.service_readiness.declared_dependencies]

## Phase 3: Consumers

- [x] [serial] T3.1 Report `started` and `ready` separately for the coordination daemon, including restart, and name its restart policy (`evidence/runtime-final-cohort-2026-10-27.json`; real subscription and manual relaunch under `never`, not automatic supervision). r[mantle.service_readiness.readiness_vocabulary]
- [x] [serial] T3.2 Report `started` and `ready` separately for the Rust cache daemon, and name its restart policy (`evidence/runtime-final-cohort-2026-10-27.json`; real compiler response, early-exit controls, and fresh manual invocation under `never`). r[mantle.service_readiness.restart_policy_matrix]
- [x] [serial] T3.3 Report `started` and `ready` separately for the remote serve binding, and name its restart policy (`evidence/runtime-final-cohort-2026-10-27.json`; accepted flushed stdio session, rejected/malformed controls, and metadata non-publication under `never`). r[mantle.service_readiness.restart_policy_matrix]
- [ ] [serial] T3.4 Declare dependency relations for the source-built fixed-point proof stages and report each stage's state under the vocabulary. r[mantle.service_readiness.declared_dependencies]
- [x] [serial] T3.5 Make `mantle doctor` publish derived readiness state while keeping existing human and JSON output unchanged (`evidence/runtime-final-cohort-2026-10-27.json`; exact ordinary/opt-in stdout equality and separate typed stderr; T4.3 remains separate). r[mantle.service_readiness.doctor_derived_state]

Runtime status: the final-cohort captures exercise coordination subscriptions,
Rust compiler response/commit, stdio acceptance/rejection, and unchanged doctor
stdout. A stdio readiness identity rejected by the real daemon was repaired;
the CLI regression now also retains `complete`/`failed` terminal rows. The
23 core, 48 wrapper, 8 stdio CLI, and 20 daemon tests passed, with affected
package formatting and root strict Clippy. Exact selected source identities
and the frozen executable are retained in the evidence; this is not an
approved full-source snapshot or frozen G1/G2 admission.

T3.4 remains open: declarations and a declaration-only six-stage blocked graph
do not supply stage-local protected/build-action reconciliation or a valid
source-built v2 receipt. `prove-source-built-mantle-fixed-point` I3–I5/V2
retain that authority. Missing `--source-profile` still fails before proof
execution; no stage completion is inferred from that negative control.


## Phase 4: Fixtures and verification

- [x] [parallel] T4.1 Add positive fixtures: `ready` only after a real request succeeds; a dependent component starts after its dependency is `ready`; each restart policy observed on a controlled exit (`evidence/runtime-final-cohort-2026-10-27.json`; actual coordination daemon, owned fixture children, all 16 exit/acknowledgment cells and requested component/group relaunches; no production supervisor claim). r[mantle.service_readiness.readiness_vocabulary]
- [x] [parallel] T4.2 Add negative fixtures: process exits before readiness reports `failed`, not `ready`; blocked dependency reports the blocker; unknown state rejected; missing policy rejected (`evidence/runtime-final-cohort-2026-10-27.json`; actual protocol/child controls and permanent core/CLI regressions). r[mantle.service_readiness.declared_dependencies]
- [ ] [parallel] T4.3 Prove doctor evidence separation: derived state is not accepted by evidence validators and does not change receipts. r[mantle.service_readiness.doctor_derived_state]
- [ ] [serial] T4.4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[mantle.service_readiness.readiness_vocabulary]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.service_readiness.doctor_derived_state]

T4.3 partial observations are retained, not accepted as completion. Actual
doctor coordination state is rejected by positive-controlled source-bundle
and portable-receipt validators; repeated diagnostic receipts are identical.
Neither builder consumed doctor output, and no current admitted authoritative
proof receipt supplies the integrated receipt-invariance positive control.
Do not replace that missing authority with a diagnostic receipt.

T4.4 retains focused quality success; final native lifecycle results are
recorded separately in the same evidence. The required full frozen G1/G2
route remains held for the administrator/operator observation decision and
source/Git-owner successor binding recorded in
`reconcile-main-api-lineages-2026-10-17/evidence/continuation-readiness-holds-2026-10-27.json`.
No admission retry, spec sync, archive, or successor publication follows from
these bounded runtime observations.
