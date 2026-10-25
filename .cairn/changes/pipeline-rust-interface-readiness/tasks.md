# Tasks: Pipeline Rust interface readiness

Proposed and externally gated. Authoring and structural gates do not satisfy any
implementation or upstream-fix task. The one-sandbox trial and native executor
are separate required stages, not interchangeable completion claims.

Implementation admission rechecked 2026-10-23: the exact selected rustc/Cargo
reject the actual early-metadata/headstart options. The pinned upstream source
still documents its unfixed incremental SVH defect and unsafe reservation
recycling. No trial/native early-interface route is enabled, no task is newly
completed, and serial/full-metadata evidence cannot qualify this lane. See
the [fresh probes and integration hook map](evidence/authoring-validation.md#current-implementation-admission-inspection-2026-10-23)
and [proposed ADR 0126](../../../adr/0126-require-qualified-providers-before-admitting-bounded-rust-sessions.md).

## Phase 1: Prerequisites and compiler admission

- [ ] [serial] T1.1 Retain the three prerequisite changes' accepted observation, differential, and bounded-session contracts; select admitted small chain/wide workloads without expanding the per-unit source-slice limit. r[rust_package_planning.phase_readiness.acceptance]
- [ ] [serial] T1.2 Admit a source-fixed Headstart-compatible compiler/provider revision with default-hash unchanged-warm, body/interface edit, break/fix, edit/revert, and clean-vs-incremental identity regressions; reject cache flushing, disabled incremental, or alternate metadata hashing as a substitute. r[rust_package_planning.phase_readiness.compiler_gate]
- [ ] [serial] T1.3 Record exact compiler/Cargo/source/patch/sysroot/runtime/policy identities, reuse terms, capability probes, failure disposition, and the local session/store boundary in a new ADR; keep missing capabilities explicitly blocked. r[rust_package_planning.phase_readiness.compiler_gate]

## Phase 2: Single-sandbox qualification

- [ ] [serial] T2.1 Add an explicit Nickel-authored opt-in trial selection to the existing offline Cargo lane, with admitted flags/config and no ambient unstable environment or default-mode change. r[rust_package_planning.phase_readiness.sandbox_trial]
- [ ] [serial] T2.2 Qualify live-wrapper/direct-path policy compatibility, strict resident/reservation/memory limits, held results, and teardown before the trial can dispatch; do not accept token-free resume as enforcement. r[rust_package_planning.phase_readiness.sandbox_trial]
- [ ] [serial] T2.3 Run retained same-binary off/on cold/warm/incremental and failure trials inside one sandbox; compare full-metadata baselines, final output authority, error latency, memory/CPU, and all failed/timed-out runs. r[rust_package_planning.phase_readiness.sandbox_trial]

## Phase 3: Native typed readiness and execution

- [ ] [serial] T3.1 Define typed exact-attempt interface/full-metadata/link-artifact requirements and final-confirmation separation in the existing pure Rust model, including conservative proc-macro/dylib/script and LTO barriers. r[rust_package_planning.phase_readiness.typed_requirements]
- [ ] [serial] T3.2 Extend the Rust application execution ports and imperative session shell for bounded live notifications and canonical readiness dispatch using the prerequisite provisional-session mechanism; retain one actual compiler invocation per unit. r[rust_package_planning.phase_readiness.execution]
- [ ] [serial] T3.3 Integrate serial and ordinary full-metadata baselines, early-interface mode capability admission, full-metadata identity checks, final-artifact waits, and existing cache/receipt finalization without changing generic goal/store semantics. r[rust_package_planning.phase_readiness.execution]

## Phase 4: Behavioral acceptance

- [ ] [parallel] T4.1 Exercise a real supported chain where dependent analysis overlaps producer body checking; capture actual phase notifications, matching full metadata before codegen, and link/LTO barriers at the declared optimization levels. r[rust_package_planning.phase_readiness.typed_requirements]
- [ ] [parallel] T4.2 Run the differential incremental/failure matrix with real producer body errors, crashes, cancellation, stale outputs, const/generic/opaque/async cases, and final incremental-vs-clean results; retain required blocked/unrun cells explicitly. r[rust_package_planning.phase_readiness.acceptance]
- [ ] [parallel] T4.3 Exercise real proc-macro and build-script consumption barriers, incompatible/buffering wrappers, malformed/wrong-attempt notifications, and metadata identity mismatches; prove no speculative manifest, cache result, or signed PathInfo is published. r[rust_package_planning.phase_readiness.execution]
- [ ] [parallel] T4.4 Exercise one-slot, saturated, and memory/elapsed/output exhaustion sessions; verify observed process-tree teardown and default path parity without oversubscription. r[rust_package_planning.phase_readiness.acceptance]

## Phase 5: Verification and acceptance

- [ ] [serial] T5.1 Run supported native/default smokes, focused first-party Rust formatting/Clippy/tests, no-std boundaries, applicable Nickel/compatibility checks, and Cairn validate plus proposal/design/tasks gates; retain exact command/input evidence. r[rust_package_planning.phase_readiness.acceptance]
- [ ] [serial] T5.2 Update operator/toolchain/cache documentation, compatibility declarations, and release notes with support cells, explicit enablement, blocked prerequisites, actual measurements and non-claims; do not switch the default or claim the whole Mantle workspace was admitted. r[rust_package_planning.phase_readiness.sandbox_trial]
- [ ] [serial] T5.3 Obtain acceptance for both the qualified trial and native executor; commit implementation before any separately authorized sync/archive and retain final lifecycle transcripts. r[rust_package_planning.phase_readiness.acceptance]
